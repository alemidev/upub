use apb::{target::Addressed, Activity, Base, Object};
use sea_orm::{sea_query::Expr, ActiveModelTrait, ActiveValue::{NotSet, Set, Unchanged}, ColumnTrait, Condition, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, SelectColumns};
use crate::{ext::{AnyQuery, LoggableError}, model, traits::{fetch::Pull, Fetcher, Normalizer}};

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

	#[error("activity {0} not processable by this application")]
	Unprocessable(String),

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
			_ => Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
		}
	}
}

pub async fn create(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let Some(object_node) = activity.object().extract() else {
		// TODO we could process non-embedded activities or arrays but im lazy rn
		tracing::error!("refusing to process activity without embedded object");
		return Err(ProcessorError::Unprocessable(activity.id()?.to_string()));
	};
	if model::object::Entity::ap_to_internal(object_node.id()?, tx).await?.is_some() {
		return Err(ProcessorError::AlreadyProcessed);
	}
	if object_node.attributed_to().id()? != activity.actor().id()? {
		return Err(ProcessorError::Unauthorized);
	}
	if let Ok(reply) = object_node.in_reply_to().id() {
		if let Err(e) = ctx.fetch_object(reply, tx).await {
			tracing::warn!("failed fetching replies for received object: {e}");
		}
	}
	let object_model = ctx.insert_object(object_node, tx).await?;
	// only likes mentioning local users are stored to generate notifications, everything else
	// produces side effects but no activity, and thus no notification
	if ctx.is_local(activity.actor().id()?) || activity.mentioning().iter().any(|x| ctx.is_local(x)) {
		ctx.insert_activity(activity, tx).await?;
	}
	tracing::debug!("{} posted {}", object_model.attributed_to.as_deref().unwrap_or("<anonymous>"), object_model.id);
	Ok(())
}

pub async fn like(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let actor = ctx.fetch_user(activity.actor().id()?, tx).await?;
	let obj = ctx.fetch_object(activity.object().id()?, tx).await?;
	if crate::model::like::Entity::find_by_uid_oid(actor.internal, obj.internal)
		.any(tx)
		.await?
	{
		return Err(ProcessorError::AlreadyProcessed);
	}

	let like = crate::model::like::ActiveModel {
		internal: NotSet,
		actor: Set(actor.internal),
		object: Set(obj.internal),
		published: Set(activity.published().unwrap_or_else(|_|chrono::Utc::now())),
	};

	crate::model::like::Entity::insert(like).exec(tx).await?;
	crate::model::object::Entity::update_many()
		.col_expr(crate::model::object::Column::Likes, Expr::col(crate::model::object::Column::Likes).add(1))
		.filter(crate::model::object::Column::Internal.eq(obj.internal))
		.exec(tx)
		.await?;

	// only likes mentioning local users are stored to generate notifications, everything else
	// produces side effects but no activity, and thus no notification
	if ctx.is_local(&actor.id) || activity.mentioning().iter().any(|x| ctx.is_local(x)) {
		ctx.insert_activity(activity, tx).await?;
	}

	tracing::debug!("{} liked {}", actor.id, obj.id);
	Ok(())
}

pub async fn follow(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let source_actor = crate::model::actor::Entity::find_by_ap_id(activity.actor().id()?)
		.one(tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;
	let target_actor = ctx.fetch_user(activity.object().id()?, tx).await?;
	let activity_model = ctx.insert_activity(activity, tx).await?;

	if let Some(relation) = crate::model::relation::Entity::find()
		.filter(crate::model::relation::Column::Follower.eq(source_actor.internal))
		.filter(crate::model::relation::Column::Following.eq(target_actor.internal))
		.select_only()
		.select_column(crate::model::relation::Column::Internal)
		.into_tuple::<i64>()
		.one(tx)
		.await?
	{
		// already requested, update pending row
		crate::model::relation::Entity::update_many()
			.col_expr(crate::model::relation::Column::Activity, Expr::value(Some(activity_model.internal)))
			.filter(crate::model::relation::Column::Internal.eq(relation))
			.exec(tx)
			.await?;

	} else {

		let follower_instance = crate::model::instance::Entity::domain_to_internal(&source_actor.domain, tx)
			.await?
			.ok_or(ProcessorError::Incomplete)?;

		let following_instance = crate::model::instance::Entity::domain_to_internal(&target_actor.domain, tx)
			.await?
			.ok_or(ProcessorError::Incomplete)?;

		// new follow request, make new row
		let relation_model = crate::model::relation::ActiveModel {
			internal: NotSet,
			accept: Set(None),
			activity: Set(activity_model.internal),
			follower: Set(source_actor.internal),
			follower_instance: Set(follower_instance),
			following: Set(target_actor.internal),
			following_instance: Set(following_instance),
		};
		crate::model::relation::Entity::insert(relation_model)
			.exec(tx).await?;
	}

	tracing::info!("{} wants to follow {}", activity_model.actor, target_actor.id);
	Ok(())
}

pub async fn accept(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	// TODO what about TentativeAccept
	let follow_activity = crate::model::activity::Entity::find_by_ap_id(activity.object().id()?)
		.one(tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	if follow_activity.object.unwrap_or_default() != activity.actor().id()? {
		return Err(ProcessorError::Unauthorized);
	}

	let activity_model = ctx.insert_activity(activity, tx).await?;

	let follower = crate::model::actor::Entity::ap_to_internal(&follow_activity.actor, tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;
	let following = crate::model::actor::Entity::ap_to_internal(&activity_model.actor, tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	crate::model::relation::Entity::update_many()
		.col_expr(crate::model::relation::Column::Accept, Expr::value(Some(activity_model.internal)))
		.col_expr(crate::model::relation::Column::Activity, Expr::value(follow_activity.internal))
		.filter(crate::model::relation::Column::Follower.eq(follower))
		.filter(crate::model::relation::Column::Following.eq(following))
		.exec(tx)
		.await?;

	crate::model::actor::Entity::update_many()
		.col_expr(
			crate::model::actor::Column::FollowingCount,
			Expr::col(crate::model::actor::Column::FollowingCount).add(1)
		)
		.filter(crate::model::actor::Column::Internal.eq(follower))
		.exec(tx)
		.await?;
	crate::model::actor::Entity::update_many()
		.col_expr(
			crate::model::actor::Column::FollowersCount,
			Expr::col(crate::model::actor::Column::FollowersCount).add(1)
		)
		.filter(crate::model::actor::Column::Internal.eq(following))
		.exec(tx)
		.await?;

	tracing::debug!("{} accepted follow request by {}", activity_model.actor, follow_activity.actor);

	Ok(())
}

pub async fn reject(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	// TODO what about TentativeReject?
	let follow_activity = crate::model::activity::Entity::find_by_ap_id(activity.object().id()?)
		.one(tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	if follow_activity.object.unwrap_or_default() != activity.actor().id()? {
		return Err(ProcessorError::Unauthorized);
	}

	let activity_model = ctx.insert_activity(activity, tx).await?;

	crate::model::relation::Entity::delete_many()
		.filter(crate::model::relation::Column::Activity.eq(activity_model.internal))
		.exec(tx)
		.await?;

	tracing::debug!("{} rejected follow request by {}", activity_model.actor, follow_activity.actor);

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
	let Some(object_node) = activity.object().extract() else {
		tracing::error!("refusing to process activity without embedded object");
		return Err(ProcessorError::Unprocessable(activity.id()?.to_string()));
	};

	let actor_id = activity.actor().id()?.to_string();
	let oid = object_node.id()?.to_string();

	match object_node.object_type()? {
		apb::ObjectType::Actor(_) => {
			if oid != actor_id {
				return Err(ProcessorError::Unauthorized);
			}
			let internal_uid = crate::model::actor::Entity::ap_to_internal(&oid, tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			let mut actor_model = crate::AP::actor_q(object_node.as_actor()?)?;
			actor_model.internal = Unchanged(internal_uid);
			actor_model.updated = Set(chrono::Utc::now());
			actor_model.update(tx).await?;
		},
		apb::ObjectType::Note | apb::ObjectType::Document(apb::DocumentType::Page) => {
			let internal_oid = crate::model::object::Entity::ap_to_internal(&oid, tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			let mut object_model = crate::AP::object_q(&object_node)?;
			object_model.internal = Unchanged(internal_oid);
			object_model.updated = Set(chrono::Utc::now());
			object_model.update(tx).await?;
		},
		_ => return Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
	}

	if ctx.is_local(&actor_id) {
		ctx.insert_activity(activity, tx).await?;
	}

	tracing::debug!("{} updated {}", actor_id, oid);
	Ok(())
}

pub async fn undo(_ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	// TODO in theory we could work with just object_id but right now only accept embedded
	let undone_activity = activity.object()
		.extract()
		.ok_or(apb::FieldErr("object"))?;

	let uid = activity.actor().id()?.to_string();
	let internal_uid = crate::model::actor::Entity::ap_to_internal(&uid, tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;

	if uid != undone_activity.as_activity()?.actor().id()? {
		return Err(ProcessorError::Unauthorized);
	}

	match undone_activity.as_activity()?.activity_type()? {
		apb::ActivityType::Like => {
			let internal_oid = crate::model::object::Entity::ap_to_internal(
				undone_activity.as_activity()?.object().id()?,
				tx
			)
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
			let internal_uid_following = crate::model::actor::Entity::ap_to_internal(
				undone_activity.as_activity()?.object().id()?,
				tx,
			)
				.await?
				.ok_or(ProcessorError::Incomplete)?;

			// no pending relation to undo
			let relation = crate::model::relation::Entity::find()
				.filter(model::relation::Column::Follower.eq(internal_uid))
				.filter(model::relation::Column::Following.eq(internal_uid_following))
				.one(tx)
				.await?
				.ok_or(ProcessorError::AlreadyProcessed)?;

			crate::model::relation::Entity::delete_many()
				.filter(crate::model::relation::Column::Follower.eq(internal_uid))
				.filter(crate::model::relation::Column::Following.eq(internal_uid_following))
				.exec(tx)
				.await?;

			if relation.accept.is_some() {
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
			}
		},
		_ => return Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
	}

	Ok(())
}

pub async fn announce(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let announced_id = activity.object().id()?.to_string();

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
				Pull::Actor(_) => return Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
				// objects are processed down below, make a mock Internal::Object(internal)
				Pull::Object(x) =>
					crate::context::Internal::Object(
						ctx.resolve_object(x, tx).await?.internal
					),
			}
		}
	} {
		crate::context::Internal::Actor(_) => Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
		crate::context::Internal::Activity(_) => Err(ProcessorError::AlreadyProcessed), // ???
		crate::context::Internal::Object(internal) => {
			let actor = ctx.fetch_user(activity.actor().id()?, tx).await?;

			// we only care about "organic" announces, as in those produced by people
			// anything shared by groups, services or applications is just mirroring: fetch it and be done
			if actor.actor_type == apb::ActorType::Person {
				let share = crate::model::announce::ActiveModel {
					internal: NotSet,
					actor: Set(actor.internal),
					object: Set(internal),
					published: Set(activity.published().unwrap_or(chrono::Utc::now())),
				};

				crate::model::announce::Entity::insert(share)
					.exec(tx).await?;

				// if this user never "boosted" this object before, increase its counter
				if !crate::model::announce::Entity::find_by_uid_oid(actor.internal, internal)
					.any(tx)
					.await?
				{
					crate::model::object::Entity::update_many()
						.col_expr(crate::model::object::Column::Announces, Expr::col(crate::model::object::Column::Announces).add(1))
						.filter(crate::model::object::Column::Internal.eq(internal))
						.exec(tx)
						.await?;
				}
			}

			// TODO we should probably insert an activity, otherwise this won't appear on timelines!!
			//      or maybe go update all addressing records for this object, pushing them up
			//      or maybe create new addressing rows with more recent dates
			//      or maybe create fake objects that reference the original one
			//      idk!!!!

			if ctx.is_local(&actor.id) {
				ctx.insert_activity(activity, tx).await?;
			}

			tracing::debug!("{} shared {}", actor.id, announced_id);
			Ok(())
		},
	}
}
