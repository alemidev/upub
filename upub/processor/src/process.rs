use apb::{target::Addressed, Activity, Base, Object};
use sea_orm::{sea_query::Expr, ActiveValue::{NotSet, Set}, ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, SelectColumns};
use upub::{errors::LoggableError, ext::AnyQuery};
use crate::{address::Addresser, fetch::{Fetcher, Pull}, normalize::Normalizer};

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
	NormalizerError(#[from] crate::normalize::NormalizerError),

	#[error("failed fetching resource: {0:?}")]
	PullError(#[from] crate::fetch::PullError),
}

#[async_trait::async_trait]
pub trait Processor {
	async fn process(&self, activity: impl apb::Activity) -> Result<(), ProcessorError>;
}

#[async_trait::async_trait]
impl Processor for upub::Context {
	async fn process(&self, activity: impl apb::Activity) -> Result<(), ProcessorError> {
		// TODO we could process Links and bare Objects maybe, but probably out of AP spec?
		match activity.activity_type()? {
			// TODO emojireacts are NOT likes, but let's process them like ones for now maybe?
			apb::ActivityType::Like | apb::ActivityType::EmojiReact => Ok(like(self, activity).await?),
			apb::ActivityType::Create => Ok(create(self, activity).await?),
			apb::ActivityType::Follow => Ok(follow(self, activity).await?),
			apb::ActivityType::Announce => Ok(announce(self, activity).await?),
			apb::ActivityType::Accept(_) => Ok(accept(self, activity).await?),
			apb::ActivityType::Reject(_) => Ok(reject(self, activity).await?),
			apb::ActivityType::Undo => Ok(undo(self, activity).await?),
			apb::ActivityType::Delete => Ok(delete(self, activity).await?),
			apb::ActivityType::Update => Ok(update(self, activity).await?),
			_ => Err(ProcessorError::Unprocessable),
		}
	}
}

pub async fn create(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	let Some(object_node) = activity.object().extract() else {
		// TODO we could process non-embedded activities or arrays but im lazy rn
		tracing::error!("refusing to process activity without embedded object");
		return Err(ProcessorError::Unprocessable);
	};
	if let Ok(reply) = object_node.in_reply_to().id() {
		if let Err(e) = ctx.fetch_object(reply).await {
			tracing::warn!("failed fetching replies for received object: {e}");
		}
	}
	let activity_model = ctx.insert_activity(activity).await?;
	let object_model = ctx.insert_object(object_node).await?;
	let expanded_addressing = ctx.expand_addressing(object_model.addressed()).await?;
	ctx.address_to(Some(activity_model.internal), Some(object_model.internal), &expanded_addressing).await?;
	tracing::info!("{} posted {}", activity_model.actor, object_model.id);
	Ok(())
}

pub async fn like(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	let internal_uid = upub::model::actor::Entity::ap_to_internal(&uid, ctx.db())
		.await?
		.ok_or(ProcessorError::Incomplete)?;
	let object_uri = activity.object().id()?.to_string();
	let published = activity.published().unwrap_or_else(|_|chrono::Utc::now());
	let obj = ctx.fetch_object(&object_uri).await?;
	if upub::model::like::Entity::find_by_uid_oid(internal_uid, obj.internal)
		.any(ctx.db())
		.await?
	{
		return Err(ProcessorError::AlreadyProcessed);
	}

	let activity_model = ctx.insert_activity(activity).await?;

	let like = upub::model::like::ActiveModel {
		internal: NotSet,
		actor: Set(internal_uid),
		object: Set(obj.internal),
		activity: Set(activity_model.internal),
		published: Set(published),
	};
	upub::model::like::Entity::insert(like).exec(ctx.db()).await?;
	upub::model::object::Entity::update_many()
		.col_expr(upub::model::object::Column::Likes, Expr::col(upub::model::object::Column::Likes).add(1))
		.filter(upub::model::object::Column::Internal.eq(obj.internal))
		.exec(ctx.db())
		.await?;

	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed()).await?;
	if expanded_addressing.is_empty() { // WHY MASTODON!!!!!!!
		expanded_addressing.push(
			upub::model::object::Entity::find_by_id(obj.internal)
				.select_only()
				.select_column(upub::model::object::Column::AttributedTo)
				.into_tuple::<String>()
				.one(ctx.db())
				.await?
				.ok_or(ProcessorError::Incomplete)?
			);
	}
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
	tracing::info!("{} liked {}", uid, obj.id);
	Ok(())
}

pub async fn follow(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	let source_actor = activity.actor().id()?.to_string();
	let source_actor_internal = upub::model::actor::Entity::ap_to_internal(&source_actor, ctx.db())
		.await?
		.ok_or(ProcessorError::Incomplete)?;
	let target_actor = activity.object().id()?.to_string();
	let usr = ctx.fetch_user(&target_actor).await?;
	let activity_model = ctx.insert_activity(activity).await?;
	let relation_model = upub::model::relation::ActiveModel {
		internal: NotSet,
		accept: Set(None),
		activity: Set(activity_model.internal),
		follower: Set(source_actor_internal),
		following: Set(usr.internal),
	};
	upub::model::relation::Entity::insert(relation_model)
		.exec(ctx.db()).await?;
	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed()).await?;
	if !expanded_addressing.contains(&target_actor) {
		expanded_addressing.push(target_actor);
	}
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
	tracing::info!("{} wants to follow {}", source_actor, usr.id);
	Ok(())
}

pub async fn accept(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	// TODO what about TentativeAccept
	let target_actor = activity.actor().id()?.to_string();
	let follow_request_id = activity.object().id()?.to_string();
	let follow_activity = upub::model::activity::Entity::find_by_ap_id(&follow_request_id)
		.one(ctx.db())
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	if follow_activity.object.unwrap_or_default() != target_actor {
		return Err(ProcessorError::Unauthorized);
	}

	let activity_model = ctx.insert_activity(activity).await?;

	upub::model::actor::Entity::update_many()
		.col_expr(
			upub::model::actor::Column::FollowingCount,
			Expr::col(upub::model::actor::Column::FollowingCount).add(1)
		)
		.filter(upub::model::actor::Column::Id.eq(&follow_activity.actor))
		.exec(ctx.db())
		.await?;
	upub::model::actor::Entity::update_many()
		.col_expr(
			upub::model::actor::Column::FollowersCount,
			Expr::col(upub::model::actor::Column::FollowersCount).add(1)
		)
		.filter(upub::model::actor::Column::Id.eq(&follow_activity.actor))
		.exec(ctx.db())
		.await?;

	upub::model::relation::Entity::update_many()
		.col_expr(upub::model::relation::Column::Accept, Expr::value(Some(activity_model.internal)))
		.filter(upub::model::relation::Column::Activity.eq(follow_activity.internal))
		.exec(ctx.db()).await?;

	tracing::info!("{} accepted follow request by {}", target_actor, follow_activity.actor);

	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed()).await?;
	if !expanded_addressing.contains(&follow_activity.actor) {
		expanded_addressing.push(follow_activity.actor);
	}
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
	Ok(())
}

pub async fn reject(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	// TODO what about TentativeReject?
	let uid = activity.actor().id()?.to_string();
	let follow_request_id = activity.object().id()?.to_string();
	let follow_activity = upub::model::activity::Entity::find_by_ap_id(&follow_request_id)
		.one(ctx.db())
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	if follow_activity.object.unwrap_or_default() != uid {
		return Err(ProcessorError::Unauthorized);
	}

	let activity_model = ctx.insert_activity(activity).await?;

	upub::model::relation::Entity::delete_many()
		.filter(upub::model::relation::Column::Activity.eq(activity_model.internal))
		.exec(ctx.db())
		.await?;

	tracing::info!("{} rejected follow request by {}", uid, follow_activity.actor);

	let mut expanded_addressing = ctx.expand_addressing(activity_model.addressed()).await?;
	if !expanded_addressing.contains(&follow_activity.actor) {
		expanded_addressing.push(follow_activity.actor);
	}

	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
	Ok(())
}

pub async fn delete(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	let oid = activity.object().id()?.to_string();
	upub::model::actor::Entity::delete_by_ap_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from users");
	upub::model::object::Entity::delete_by_ap_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from objects");
	tracing::debug!("deleted '{oid}'");
	Ok(())
}

pub async fn update(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	let aid = activity.id()?.to_string();
	let Some(object_node) = activity.object().extract() else {
		tracing::error!("refusing to process activity without embedded object");
		return Err(ProcessorError::Unprocessable);
	};
	let oid = object_node.id()?.to_string();

	let activity_model = ctx.insert_activity(activity).await?;

	match object_node.object_type()? {
		apb::ObjectType::Actor(_) => {
			let internal_uid = upub::model::actor::Entity::ap_to_internal(&oid, ctx.db())
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			let mut actor_model = upub::model::actor::ActiveModel::new(object_node.as_actor()?)?;
			actor_model.internal = Set(internal_uid);
			actor_model.updated = Set(chrono::Utc::now());
			upub::model::actor::Entity::update(actor_model)
				.exec(ctx.db())
				.await?;
		},
		apb::ObjectType::Note => {
			let internal_oid = upub::model::object::Entity::ap_to_internal(&oid, ctx.db())
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			let mut object_model = upub::model::object::ActiveModel::new(&object_node)?;
			object_model.internal = Set(internal_oid);
			object_model.updated = Set(chrono::Utc::now());
			upub::model::object::Entity::update(object_model)
				.exec(ctx.db())
				.await?;
		},
		t => tracing::warn!("no side effects implemented for update type {t:?}"),
	}

	tracing::info!("{} updated {}", uid, oid);
	let expanded_addressing = ctx.expand_addressing(activity_model.addressed()).await?;
	ctx.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
	Ok(())
}

pub async fn undo(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	// TODO in theory we could work with just object_id but right now only accept embedded
	let undone_activity = activity.object().extract().ok_or(apb::FieldErr("object"))?;
	let undone_activity_id = undone_activity.id()?;
	let undone_activity_author = undone_activity.as_activity()?.actor().id()?.to_string();

	if uid != undone_activity_author {
		return Err(ProcessorError::Unauthorized);
	}

	let undone_activity_target = undone_activity.as_activity()?.object().id()?.to_string();

	let internal_uid = upub::model::actor::Entity::ap_to_internal(&uid, ctx.db())
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	let activity_type = activity.activity_type()?;
	let targets = ctx.expand_addressing(activity.addressed()).await?;
	let activity_model = ctx.insert_activity(activity).await?;
	ctx.address_to(Some(activity_model.internal), None, &targets).await?;

	match activity_type {
		apb::ActivityType::Like => {
			let internal_oid = upub::model::object::Entity::ap_to_internal(&undone_activity_target, ctx.db())
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			upub::model::like::Entity::delete_many()
				.filter(
					Condition::all()
						.add(upub::model::like::Column::Actor.eq(internal_uid))
						.add(upub::model::like::Column::Object.eq(internal_oid))
				)
				.exec(ctx.db())
				.await?;
			upub::model::object::Entity::update_many()
				.filter(upub::model::object::Column::Internal.eq(internal_oid))
				.col_expr(upub::model::object::Column::Likes, Expr::col(upub::model::object::Column::Likes).sub(1))
				.exec(ctx.db())
				.await?;
		},
		apb::ActivityType::Follow => {
			let internal_uid_following = upub::model::actor::Entity::ap_to_internal(&undone_activity_target, ctx.db())
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			upub::model::relation::Entity::delete_many()
				.filter(upub::model::relation::Column::Follower.eq(internal_uid))
				.filter(upub::model::relation::Column::Following.eq(internal_uid_following))
				.exec(ctx.db())
				.await?;
			upub::model::actor::Entity::update_many()
				.filter(upub::model::actor::Column::Internal.eq(internal_uid))
				.col_expr(upub::model::actor::Column::FollowingCount, Expr::col(upub::model::actor::Column::FollowingCount).sub(1))
				.exec(ctx.db())
				.await?;
			upub::model::actor::Entity::update_many()
				.filter(upub::model::actor::Column::Internal.eq(internal_uid_following))
				.col_expr(upub::model::actor::Column::FollowersCount, Expr::col(upub::model::actor::Column::FollowersCount).sub(1))
				.exec(ctx.db())
				.await?;
		},
		t => {
			tracing::error!("received 'Undo' for unimplemented activity type: {t:?}");
			return Err(ProcessorError::Unprocessable);
		},
	}

	Ok(())
}

pub async fn announce(ctx: &upub::Context, activity: impl apb::Activity) -> Result<(), ProcessorError> {
	let uid = activity.actor().id()?.to_string();
	let actor = ctx.fetch_user(&uid).await?;
	let internal_uid = upub::model::actor::Entity::ap_to_internal(&uid, ctx.db())
		.await?
		.ok_or(ProcessorError::Incomplete)?;
	let announced_id = activity.object().id()?.to_string();
	let published = activity.published().unwrap_or(chrono::Utc::now());
	let addressed = activity.addressed();
	
	match ctx.pull(&announced_id).await? {
		Pull::Actor(_) => Err(ProcessorError::Unprocessable),
		Pull::Object(object) => {

			let object_model = ctx.resolve_object(object).await?;
			let activity_model = ctx.insert_activity(activity).await?;

			// relays send us objects as Announce, but we don't really want to count those towards the
			// total shares count of an object, so just fetch the object and be done with it
			if !matches!(actor.actor_type, apb::ActorType::Person) {
				tracing::info!("relay {} broadcasted {}", activity_model.actor, announced_id);
				return Ok(())
			}

			let share = upub::model::announce::ActiveModel {
				internal: NotSet,
				actor: Set(internal_uid),
				object: Set(object_model.internal),
				published: Set(published),
			};

			let expanded_addressing = ctx.expand_addressing(addressed).await?;
			ctx.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
			upub::model::announce::Entity::insert(share)
				.exec(ctx.db()).await?;
			upub::model::object::Entity::update_many()
				.col_expr(upub::model::object::Column::Announces, Expr::col(upub::model::object::Column::Announces).add(1))
				.filter(upub::model::object::Column::Internal.eq(object_model.internal))
				.exec(ctx.db())
				.await?;

			tracing::info!("{} shared {}", activity_model.actor, announced_id);
			Ok(())
		},
		Pull::Activity(activity) => {
			// groups update all members of other things that happen inside, process those
			match activity.activity_type()? {
				apb::ActivityType::Like | apb::ActivityType::EmojiReact => Ok(like(ctx, activity).await?),
				apb::ActivityType::Create => Ok(create(ctx, activity).await?),
				apb::ActivityType::Undo => Ok(undo(ctx, activity).await?),
				apb::ActivityType::Delete => Ok(delete(ctx, activity).await?),
				apb::ActivityType::Update => Ok(update(ctx, activity).await?),
				x => {
					tracing::warn!("ignoring unhandled announced activity of type {x:?}");
					Err(ProcessorError::Unprocessable)
				},
			}
		},
	}
}
