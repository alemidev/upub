use apb::{target::Addressed, Activity, Base, Object};
use sea_orm::{sea_query::Expr, ActiveValue::{NotSet, Set}, ColumnTrait, Condition, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, SelectColumns};
use crate::{ext::{AnyQuery, LoggableError}, model, traits::{fetch::Pull, Addresser, Fetcher, Normalizer}};

#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
	#[error("activity already processed")]
	AlreadyProcessed,

	#[error("processed activity misses required field: '{0}'")]
	Malformed(#[from] apb::FieldErr),

	#[error("database error while processing: {0:?}")]
	DbErr(#[from] sea_orm::DbErr),

	#[error("actor is not authorized to carry out this activity")]
	Unauthorized,

	#[error("could not resolve all objects involved in this activity")]
	Incomplete,

	#[error("activity not processable by this application")]
	Unprocessable,

	#[error("failed normalizing and inserting entity: {0:?}")]
	NormalizerError(#[from] crate::traits::normalize::NormalizerError),

	#[error("failed fetching resource: {0:?}")]
	PullError(#[from] crate::traits::fetch::PullError),
}

#[async_trait::async_trait]
pub trait Processor {
	async fn process(&self, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError>;
}

#[async_trait::async_trait]
impl Processor for crate::Context {
	async fn process(&self, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
		// TODO we could process Links and bare Objects maybe, but probably out of AP spec?
		match activity.activity_type()? {
			// TODO emojireacts are NOT likes, but let's process them like ones for now maybe?
			apb::ActivityType::Like | apb::ActivityType::EmojiReact => Ok(like(self, activity, tx).await?),
			apb::ActivityType::Create => Ok(create(self, activity, tx).await?),
			apb::ActivityType::Follow => Ok(follow(self, activity, tx).await?),
			apb::ActivityType::Announce => Ok(announce(self, activity, tx).await?),
			apb::ActivityType::Accept(_) => Ok(accept(self, activity, tx).await?),
			apb::ActivityType::Reject(_) => Ok(reject(self, activity, tx).await?),
			apb::ActivityType::Undo => Ok(undo(self, activity, tx).await?),
			apb::ActivityType::Delete => Ok(delete(self, activity, tx).await?),
			apb::ActivityType::Update => Ok(update(self, activity, tx).await?),
			_ => Err(ProcessorError::Unprocessable),
		}
	}
}

pub async fn create(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let Some(object_node) = activity.object().extract() else {
		// TODO we could process non-embedded activities or arrays but im lazy rn
		tracing::error!("refusing to process activity without embedded object");
		return Err(ProcessorError::Unprocessable);
	};
	let oid = object_node.id()?.to_string();
	let addressed = object_node.addressed();
	let activity_model = ctx.insert_activity(activity, tx).await?;
	let internal_oid = if let Some(internal) = model::object::Entity::ap_to_internal(&oid, tx).await? {
		tracing::debug!("skipping insertion of already known object #{internal}");
		internal
	} else {
		if let Ok(reply) = object_node.in_reply_to().id() {
			if let Err(e) = ctx.fetch_object(reply, tx).await {
				tracing::warn!("failed fetching replies for received object: {e}");
			}
		}
		let object_model = ctx.insert_object(object_node, tx).await?;
		object_model.internal
	};
	let expanded_addressing = ctx.expand_addressing(addressed, tx).await?;
	ctx.address_to(Some(activity_model.internal), Some(internal_oid), &expanded_addressing, tx).await?;
	tracing::info!("{} posted {}", activity_model.actor, oid);
	Ok(())
}

pub async fn like(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	let actor = ctx.fetch_user(&uid, tx).await?;
	let object_uri = activity.object().id()?.to_string();
	let published = activity.published().unwrap_or_else(|_|chrono::Utc::now());
	let obj = ctx.fetch_object(&object_uri, tx).await?;
	if crate::model::like::Entity::find_by_uid_oid(actor.internal, obj.internal)
		.any(tx)
		.await?
	{
		return Err(ProcessorError::AlreadyProcessed);
	}

	let activity_model = ctx.insert_activity(activity, tx).await?;

	let like = crate::model::like::ActiveModel {
		internal: NotSet,
		actor: Set(actor.internal),
		object: Set(obj.internal),
		activity: Set(activity_model.internal),
		published: Set(published),
	};
	crate::model::like::Entity::insert(like).exec(tx).await?;
	crate::model::object::Entity::update_many()
		.col_expr(crate::model::object::Column::Likes, Expr::col(crate::model::object::Column::Likes).add(1))
		.filter(crate::model::object::Column::Internal.eq(obj.internal))
		.exec(tx)
		.await?;

	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed(), tx).await?;
	if expanded_addressing.is_empty() { // WHY MASTODON!!!!!!!
		expanded_addressing.push(
			crate::model::object::Entity::find_by_id(obj.internal)
				.select_only()
				.select_column(crate::model::object::Column::AttributedTo)
				.into_tuple::<String>()
				.one(tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?
			);
	}
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing, tx).await?;
	tracing::info!("{} liked {}", uid, obj.id);
	Ok(())
}

pub async fn follow(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let source_actor = activity.actor().id()?.to_string();
	let source_actor_internal = crate::model::actor::Entity::ap_to_internal(&source_actor, tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;
	let target_actor = activity.object().id()?.to_string();
	let usr = ctx.fetch_user(&target_actor, tx).await?;
	let activity_model = ctx.insert_activity(activity, tx).await?;
	let relation_model = crate::model::relation::ActiveModel {
		internal: NotSet,
		accept: Set(None),
		activity: Set(activity_model.internal),
		follower: Set(source_actor_internal),
		following: Set(usr.internal),
	};
	crate::model::relation::Entity::insert(relation_model)
		.exec(tx).await?;
	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed(), tx).await?;
	if !expanded_addressing.contains(&target_actor) {
		expanded_addressing.push(target_actor);
	}
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing, tx).await?;
	tracing::info!("{} wants to follow {}", source_actor, usr.id);
	Ok(())
}

pub async fn accept(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	// TODO what about TentativeAccept
	let target_actor = activity.actor().id()?.to_string();
	let follow_request_id = activity.object().id()?.to_string();
	let follow_activity = crate::model::activity::Entity::find_by_ap_id(&follow_request_id)
		.one(tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	if follow_activity.object.unwrap_or_default() != target_actor {
		return Err(ProcessorError::Unauthorized);
	}

	let activity_model = ctx.insert_activity(activity, tx).await?;

	crate::model::actor::Entity::update_many()
		.col_expr(
			crate::model::actor::Column::FollowingCount,
			Expr::col(crate::model::actor::Column::FollowingCount).add(1)
		)
		.filter(crate::model::actor::Column::Id.eq(&follow_activity.actor))
		.exec(tx)
		.await?;
	crate::model::actor::Entity::update_many()
		.col_expr(
			crate::model::actor::Column::FollowersCount,
			Expr::col(crate::model::actor::Column::FollowersCount).add(1)
		)
		.filter(crate::model::actor::Column::Id.eq(&follow_activity.actor))
		.exec(tx)
		.await?;

	crate::model::relation::Entity::update_many()
		.col_expr(crate::model::relation::Column::Accept, Expr::value(Some(activity_model.internal)))
		.filter(crate::model::relation::Column::Activity.eq(follow_activity.internal))
		.exec(tx).await?;

	tracing::info!("{} accepted follow request by {}", target_actor, follow_activity.actor);

	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed(), tx).await?;
	if !expanded_addressing.contains(&follow_activity.actor) {
		expanded_addressing.push(follow_activity.actor);
	}
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing, tx).await?;
	Ok(())
}

pub async fn reject(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	// TODO what about TentativeReject?
	let uid = activity.actor().id()?.to_string();
	let follow_request_id = activity.object().id()?.to_string();
	let follow_activity = crate::model::activity::Entity::find_by_ap_id(&follow_request_id)
		.one(tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	if follow_activity.object.unwrap_or_default() != uid {
		return Err(ProcessorError::Unauthorized);
	}

	let activity_model = ctx.insert_activity(activity, tx).await?;

	crate::model::relation::Entity::delete_many()
		.filter(crate::model::relation::Column::Activity.eq(activity_model.internal))
		.exec(tx)
		.await?;

	tracing::info!("{} rejected follow request by {}", uid, follow_activity.actor);

	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed(), tx).await?;
	if !expanded_addressing.contains(&follow_activity.actor) {
		expanded_addressing.push(follow_activity.actor);
	}

	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing, tx).await?;
	Ok(())
}

pub async fn delete(_ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let oid = activity.object().id()?.to_string();
	crate::model::actor::Entity::delete_by_ap_id(&oid).exec(tx).await.info_failed("failed deleting from users");
	crate::model::object::Entity::delete_by_ap_id(&oid).exec(tx).await.info_failed("failed deleting from objects");
	tracing::debug!("deleted '{oid}'");
	Ok(())
}

pub async fn update(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	let Some(object_node) = activity.object().extract() else {
		tracing::error!("refusing to process activity without embedded object");
		return Err(ProcessorError::Unprocessable);
	};
	let oid = object_node.id()?.to_string();

	let activity_model = ctx.insert_activity(activity, tx).await?;

	match object_node.object_type()? {
		apb::ObjectType::Actor(_) => {
			let internal_uid = crate::model::actor::Entity::ap_to_internal(&oid, tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			let mut actor_model = crate::AP::actor_q(object_node.as_actor()?)?;
			actor_model.internal = Set(internal_uid);
			actor_model.updated = Set(chrono::Utc::now());
			crate::model::actor::Entity::update(actor_model)
				.exec(tx)
				.await?;
		},
		apb::ObjectType::Note => {
			let internal_oid = crate::model::object::Entity::ap_to_internal(&oid, tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			let mut object_model = crate::AP::object_q(&object_node)?;
			object_model.internal = Set(internal_oid);
			object_model.updated = Set(chrono::Utc::now());
			crate::model::object::Entity::update(object_model)
				.exec(tx)
				.await?;
		},
		t => tracing::warn!("no side effects implemented for update type {t:?}"),
	}

	tracing::info!("{} updated {}", uid, oid);
	let expanded_addressing = ctx.expand_addressing(activity_model.addressed(), tx).await?;
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing, tx).await?;
	Ok(())
}

pub async fn undo(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	// TODO in theory we could work with just object_id but right now only accept embedded
	let undone_activity = activity.object().extract().ok_or(apb::FieldErr("object"))?;
	let undone_activity_author = undone_activity.as_activity()?.actor().id()?.to_string();

	if uid != undone_activity_author {
		return Err(ProcessorError::Unauthorized);
	}

	let undone_activity_target = undone_activity.as_activity()?.object().id()?.to_string();

	let internal_uid = crate::model::actor::Entity::ap_to_internal(&uid, tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	let activity_type = activity.activity_type()?;
	let targets = ctx.expand_addressing(activity.addressed(), tx).await?;
	let activity_model = ctx.insert_activity(activity, tx).await?;
	ctx.address_to(Some(activity_model.internal), None, &targets, tx).await?;

	match activity_type {
		apb::ActivityType::Like => {
			let internal_oid = crate::model::object::Entity::ap_to_internal(&undone_activity_target, tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			crate::model::like::Entity::delete_many()
				.filter(
					Condition::all()
						.add(crate::model::like::Column::Actor.eq(internal_uid))
						.add(crate::model::like::Column::Object.eq(internal_oid))
				)
				.exec(tx)
				.await?;
			crate::model::object::Entity::update_many()
				.filter(crate::model::object::Column::Internal.eq(internal_oid))
				.col_expr(crate::model::object::Column::Likes, Expr::col(crate::model::object::Column::Likes).sub(1))
				.exec(tx)
				.await?;
		},
		apb::ActivityType::Follow => {
			let internal_uid_following = crate::model::actor::Entity::ap_to_internal(&undone_activity_target, tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			crate::model::relation::Entity::delete_many()
				.filter(crate::model::relation::Column::Follower.eq(internal_uid))
				.filter(crate::model::relation::Column::Following.eq(internal_uid_following))
				.exec(tx)
				.await?;
			crate::model::actor::Entity::update_many()
				.filter(crate::model::actor::Column::Internal.eq(internal_uid))
				.col_expr(crate::model::actor::Column::FollowingCount, Expr::col(crate::model::actor::Column::FollowingCount).sub(1))
				.exec(tx)
				.await?;
			crate::model::actor::Entity::update_many()
				.filter(crate::model::actor::Column::Internal.eq(internal_uid_following))
				.col_expr(crate::model::actor::Column::FollowersCount, Expr::col(crate::model::actor::Column::FollowersCount).sub(1))
				.exec(tx)
				.await?;
		},
		t => {
			tracing::error!("received 'Undo' for unimplemented activity type: {t:?}");
			return Err(ProcessorError::Unprocessable);
		},
	}

	Ok(())
}

pub async fn announce(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	let actor = ctx.fetch_user(&uid, tx).await?;
	let announced_id = activity.object().id()?.to_string();
	let published = activity.published().unwrap_or(chrono::Utc::now());
	let addressed = activity.addressed();

	match match ctx.find_internal(&announced_id).await? {
		// if we already have this activity, skip it
		Some(crate::context::Internal::Activity(_)) => return Ok(()), // already processed
		// actors and objects which we already have
		Some(x) => x,
		// something new, fetch it!
		None => {
			match ctx.pull(&announced_id).await? {
				// if we receive a remote activity, process it directly
				Pull::Activity(x) => return ctx.process(x, tx).await,
				// actors are not processable at all
				Pull::Actor(_) => return Err(ProcessorError::Unprocessable),
				// objects are processed down below, make a mock Internal::Object(internal)
				Pull::Object(x) =>
					crate::context::Internal::Object(
						ctx.resolve_object(x, tx).await?.internal
					),
			}
		}
	} {
		crate::context::Internal::Actor(_) => Err(ProcessorError::Unprocessable),
		crate::context::Internal::Activity(_) => Err(ProcessorError::AlreadyProcessed), // ???
		crate::context::Internal::Object(internal) => {
			let object_model = model::object::Entity::find_by_id(internal)
				.one(tx)
				.await?
				.ok_or_else(|| sea_orm::DbErr::RecordNotFound(internal.to_string()))?;
			let activity_model = ctx.insert_activity(activity, tx).await?;

			// relays send us objects as Announce, but we don't really want to count those towards the
			// total shares count of an object, so just fetch the object and be done with it
			if !matches!(actor.actor_type, apb::ActorType::Person) {
				tracing::info!("relay {} broadcasted {}", activity_model.actor, announced_id);
				return Ok(())
			}

			let share = crate::model::announce::ActiveModel {
				internal: NotSet,
				actor: Set(actor.internal),
				object: Set(object_model.internal),
				published: Set(published),
			};

			let expanded_addressing = ctx.expand_addressing(addressed, tx).await?;
			ctx.address_to(Some(activity_model.internal), None, &expanded_addressing, tx).await?;
			crate::model::announce::Entity::insert(share)
				.exec(tx).await?;
			crate::model::object::Entity::update_many()
				.col_expr(crate::model::object::Column::Announces, Expr::col(crate::model::object::Column::Announces).add(1))
				.filter(crate::model::object::Column::Internal.eq(object_model.internal))
				.exec(tx)
				.await?;

			tracing::info!("{} shared {}", activity_model.actor, announced_id);
			Ok(())
		},
	}
}
