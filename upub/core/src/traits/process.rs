use apb::{target::Addressed, Activity, Base, Object};
use sea_orm::{sea_query::Expr, ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, Condition, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, SelectColumns};
use crate::{ext::{AnyQuery, LoggableError}, model, traits::{fetch::Pull, Addresser, Cloaker, Fetcher, Normalizer}};

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
	PullError(#[from] crate::traits::fetch::RequestError),
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
			apb::ActivityType::Like => Ok(like(self, activity, tx).await?),
			apb::ActivityType::Dislike => Ok(dislike(self, activity, tx).await?),
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

	let notified = object_node.tag()
		.flat()
		.into_iter()
		.filter_map(|x| Some(x.id().ok()?.to_string()))
		.collect::<Vec<String>>();

	let object_model = ctx.insert_object(object_node, tx).await?;
	let activity_model = ctx.insert_activity(activity, tx).await?;
	ctx.address((Some(&activity_model), Some(&object_model)), tx).await?;

	for uid in notified {
		if !ctx.is_local(&uid) { continue }
		if let Some(actor_internal) = crate::model::actor::Entity::ap_to_internal(&uid, tx).await? {
			crate::Query::notify(activity_model.internal, actor_internal)
				.exec(tx)
				.await?;
		}
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

	// likes without addressing are "silent likes", process them but dont store activity or notify
	if !activity.addressed().is_empty() {
		let activity_model = ctx.insert_activity(activity, tx).await?;
		ctx.address((Some(&activity_model), None), tx).await?;

		// TODO check that object author is in this like addressing!!! otherwise skip notification
		if let Some(ref attributed_to) = obj.attributed_to {
			if ctx.is_local(attributed_to) {
				if let Some(actor_internal) = crate::model::actor::Entity::ap_to_internal(attributed_to, tx).await? {
					crate::Query::notify(activity_model.internal, actor_internal)
						.exec(tx)
						.await?;
				}
			}
		}
	}

	tracing::debug!("{} liked {}", actor.id, obj.id);
	Ok(())
}

// TODO basically same as like, can we make one function, maybe with const generic???
pub async fn dislike(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let actor = ctx.fetch_user(activity.actor().id()?, tx).await?;
	let obj = ctx.fetch_object(activity.object().id()?, tx).await?;
	if crate::model::dislike::Entity::find_by_uid_oid(actor.internal, obj.internal)
		.any(tx)
		.await?
	{
		return Err(ProcessorError::AlreadyProcessed);
	}

	let dislike = crate::model::dislike::ActiveModel {
		internal: NotSet,
		actor: Set(actor.internal),
		object: Set(obj.internal),
		published: Set(activity.published().unwrap_or_else(|_|chrono::Utc::now())),
	};

	crate::model::dislike::Entity::insert(dislike).exec(tx).await?;

	// dislikes without addressing are "silent dislikes", process them but dont store activity
	if !activity.addressed().is_empty() {
		let activity_model = ctx.insert_activity(activity, tx).await?;
		ctx.address((Some(&activity_model), None), tx).await?;

		// TODO check that object author is in this like addressing!!! otherwise skip notification
		if let Some(ref attributed_to) = obj.attributed_to {
			if ctx.is_local(attributed_to) {
				if let Some(actor_internal) = crate::model::actor::Entity::ap_to_internal(attributed_to, tx).await? {
					crate::Query::notify(activity_model.internal, actor_internal)
						.exec(tx)
						.await?;
				}
			}
		}
	}

	tracing::debug!("{} disliked {}", actor.id, obj.id);
	Ok(())
}

pub async fn follow(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let source_actor = crate::model::actor::Entity::find_by_ap_id(activity.actor().id()?)
		.one(tx)
		.await?
		.ok_or(ProcessorError::Incomplete)?;
	let target_actor = ctx.fetch_user(activity.object().id()?, tx).await?;
	let activity_model = ctx.insert_activity(activity, tx).await?;
	ctx.address((Some(&activity_model), None), tx).await?;

	if ctx.is_local(&target_actor.id) {
		crate::Query::notify(activity_model.internal, target_actor.internal)
			.exec(tx)
			.await?;
	}

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
	ctx.address((Some(&activity_model), None), tx).await?;

	if ctx.is_local(&follow_activity.actor) {
		if let Some(actor_internal) = crate::model::actor::Entity::ap_to_internal(&follow_activity.actor, tx).await? {
			crate::Query::notify(activity_model.internal, actor_internal)
				.exec(tx)
				.await?;
		}
	}

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
	ctx.address((Some(&activity_model), None), tx).await?;

	// TODO most software doesn't show this, but i think instead we should?? if someone rejects it's
	// better to know it clearly rather than not knowing if it got lost and maybe retry (being more
	// annoying)
	if ctx.is_local(&follow_activity.actor) {
		if let Some(actor_internal) = crate::model::actor::Entity::ap_to_internal(&follow_activity.actor, tx).await? {
			crate::Query::notify(activity_model.internal, actor_internal)
				.exec(tx)
				.await?;
		}
	}

	crate::model::relation::Entity::delete_many()
		.filter(crate::model::relation::Column::Activity.eq(activity_model.internal))
		.exec(tx)
		.await?;

	tracing::debug!("{} rejected follow request by {}", activity_model.actor, follow_activity.actor);

	Ok(())
}

pub async fn delete(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let oid = activity.object().id()?.to_string();
	crate::model::actor::Entity::delete_by_ap_id(&oid).exec(tx).await.info_failed("failed deleting from users");
	crate::model::object::Entity::delete_by_ap_id(&oid).exec(tx).await.info_failed("failed deleting from objects");
	// we should store deletes to make local delete deliveries work
	// except when they have empty addressing
	// so that also remote "secret" deletes dont get stored
	if !activity.addressed().is_empty() {
		let activity_model = ctx.insert_activity(activity, tx).await?;
		ctx.address((Some(&activity_model), None), tx).await?;
	}
	// TODO we should delete notifications from CREATEs related to objects we deleted
	tracing::debug!("deleted '{oid}'");
	Ok(())
}

pub async fn update(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	// TODO when attachments get updated we do nothing!!!!!!!!!!
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
			let mut actor_model = crate::AP::actor_q(object_node.as_actor()?, Some(internal_uid))?;
			if let Set(Some(ref image)) = actor_model.image {
				if !image.starts_with(ctx.base()) {
					actor_model.image = Set(Some(ctx.cloaked(image)));
				}
			}

			if let Set(Some(ref icon)) = actor_model.icon {
				if !icon.starts_with(ctx.base()) {
					actor_model.icon = Set(Some(ctx.cloaked(icon)));
				}
			}
			actor_model.updated = Set(chrono::Utc::now());
			actor_model.update(tx).await?;
		},
		apb::ObjectType::Note | apb::ObjectType::Document(apb::DocumentType::Page) => {
			let internal_oid = crate::model::object::Entity::ap_to_internal(&oid, tx)
				.await?
				.ok_or(ProcessorError::Incomplete)?;
			let mut object_model = crate::AP::object_q(&object_node, Some(internal_oid))?;
			if let Set(Some(ref content)) = object_model.content {
				object_model.content = Set(Some(ctx.sanitize(content)));
			}
			object_model.context = NotSet; // TODO dont overwrite context when updating!!
			object_model.updated = Set(chrono::Utc::now());
			object_model.update(tx).await?;
		},
		_ => return Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
	}

	// updates can be silently discarded except if local. we dont really care about knowing when
	// remote documents change, there's the "updated" field, just want the most recent version
	if ctx.is_local(&actor_id) {
		let activity_model = ctx.insert_activity(activity, tx).await?;
		ctx.address((Some(&activity_model), None), tx).await?;
	}

	tracing::debug!("{} updated {}", actor_id, oid);
	Ok(())
}

pub async fn undo(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
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

	// TODO we should store undos to make local delete deliveries work and relations make sense
	// except when they have empty addressing
	// so that also remote "secret" undos dont get stored
	if !activity.addressed().is_empty() {
		let activity_model = ctx.insert_activity(activity, tx).await?;
		ctx.address((Some(&activity_model), None), tx).await?;
	}

	if let Some(internal) = crate::model::activity::Entity::ap_to_internal(undone_activity.id()?, tx).await? {
		crate::model::notification::Entity::delete_many()
			.filter(crate::model::notification::Column::Activity.eq(internal))
			.exec(tx)
			.await?;
	}

	Ok(())
}

pub async fn announce(ctx: &crate::Context, activity: impl apb::Activity, tx: &DatabaseTransaction) -> Result<(), ProcessorError> {
	let announced_id = activity.object().id()?.to_string();

	let object = match ctx.find_internal(&announced_id).await? {
		// if we already have this activity, skip it
		Some(crate::context::Internal::Activity(_)) => return Ok(()), // already processed
		// actors and objects which we already have
		Some(crate::context::Internal::Actor(_)) => return Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
		// objects that we already have
		Some(crate::context::Internal::Object(internal)) => {
			crate::model::object::Entity::find_by_id(internal)
				.one(tx)
				.await?
				.ok_or_else(|| sea_orm::DbErr::RecordNotFound(format!("object#{internal}")))?
		},
		// something new, fetch it!
		None => {
			match ctx.pull(&announced_id).await? {
				// if we receive a remote activity, process it directly
				Pull::Activity(x) => return ctx.process(x, tx).await,
				// actors are not processable at all
				Pull::Actor(_) => return Err(ProcessorError::Unprocessable(activity.id()?.to_string())),
				// objects are processed down below, make a mock Internal::Object(internal)
				Pull::Object(x) => ctx.resolve_object(x, tx).await?,
			}
		}
	};

	let actor = ctx.fetch_user(activity.actor().id()?, tx).await?;

	// we only care about announces produced by "Person" actors, because there's intention
	// anything shared by groups, services or applications is automated: fetch it and be done
	if actor.actor_type == apb::ActorType::Person {
		// if this user never "boosted" this object before, increase its counter
		if !crate::model::announce::Entity::find_by_uid_oid(actor.internal, object.internal)
			.any(tx)
			.await?
		{
			crate::model::object::Entity::update_many()
				.col_expr(crate::model::object::Column::Announces, Expr::col(crate::model::object::Column::Announces).add(1))
				.filter(crate::model::object::Column::Internal.eq(object.internal))
				.exec(tx)
				.await?;
		}

		let share = crate::model::announce::ActiveModel {
			internal: NotSet,
			actor: Set(actor.internal),
			object: Set(object.internal),
			published: Set(activity.published().unwrap_or(chrono::Utc::now())),
		};

		crate::model::announce::Entity::insert(share)
			.exec(tx).await?;
	}

	// TODO we should probably insert an activity, otherwise this won't appear on timelines!!
	//      or maybe go update all addressing records for this object, pushing them up
	//      or maybe create new addressing rows with more recent dates
	//      or maybe create fake objects that reference the original one
	//      idk!!!!
	if actor.actor_type == apb::ActorType::Person || ctx.is_local(&actor.id) {
		let activity_model = ctx.insert_activity(activity, tx).await?;
		ctx.address((Some(&activity_model), None), tx).await?;

		if let Some(ref attributed_to) = object.attributed_to {
			if ctx.is_local(attributed_to) {
				if let Some(actor_internal) = crate::model::actor::Entity::ap_to_internal(attributed_to, tx).await? {
					crate::Query::notify(activity_model.internal, actor_internal)
						.exec(tx)
						.await?;
				}
			}
		}
	}

	tracing::debug!("{} shared {}", actor.id, announced_id);
	Ok(())
}
