use apb::{target::Addressed, Activity, Base, Object};
use reqwest::StatusCode;
use sea_orm::{sea_query::Expr, ActiveValue::{Set, NotSet}, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, SelectColumns};

use crate::{errors::{LoggableError, UpubError}, model, server::{addresser::Addresser, builders::AnyQuery, normalizer::Normalizer}};

use super::{fetcher::{Fetcher, PullResult}, side_effects::SideEffects, Context};


#[axum::async_trait]
impl apb::server::Inbox for Context {
	type Error = UpubError;
	type Activity = serde_json::Value;

	async fn create(&self, server: String, activity: serde_json::Value) -> crate::Result<()> {
		let Some(object_node) = activity.object().extract() else {
			// TODO we could process non-embedded activities or arrays but im lazy rn
			tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&activity).unwrap());
			return Err(UpubError::unprocessable());
		};
		if let Some(reply) = object_node.in_reply_to().id() {
			if let Err(e) = self.fetch_object(&reply).await {
				tracing::warn!("failed fetching replies for received object: {e}");
			}
		}
		let activity_model = self.insert_activity(activity, Some(server.clone())).await?;
		let object_model = self.insert_object(object_node, Some(server)).await?;
		let expanded_addressing = self.expand_addressing(activity_model.addressed()).await?;
		self.address_to(Some(activity_model.internal), Some(object_model.internal), &expanded_addressing).await?;
		tracing::info!("{} posted {}", activity_model.actor, object_model.id);
		Ok(())
	}

	async fn like(&self, server: String, activity: serde_json::Value) -> crate::Result<()> {
		let uid = activity.actor().id().ok_or(UpubError::bad_request())?;
		let internal_uid = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;
		let object_uri = activity.object().id().ok_or(UpubError::bad_request())?;
		let obj = self.fetch_object(&object_uri).await?;
		if model::like::Entity::find_by_uid_oid(internal_uid, obj.internal)
			.any(self.db())
			.await?
		{
			return Err(UpubError::not_modified());
		}

		let activity_model = self.insert_activity(activity, Some(server)).await?;
		self.process_like(internal_uid, obj.internal, activity_model.internal, activity_model.published).await?;
		let mut expanded_addressing = self.expand_addressing(activity_model.addressed()).await?;
		if expanded_addressing.is_empty() { // WHY MASTODON!!!!!!!
			expanded_addressing.push(
				model::object::Entity::find_by_id(obj.internal)
					.select_only()
					.select_column(model::object::Column::AttributedTo)
					.into_tuple::<String>()
					.one(self.db())
					.await?
					.ok_or_else(UpubError::not_found)?
				);
		}
		self.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
		tracing::info!("{} liked {}", uid, obj.id);
		Ok(())
	}

	async fn follow(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		let aid = activity.id().ok_or_else(UpubError::bad_request)?.to_string();
		let source_actor = activity.actor().id().ok_or_else(UpubError::bad_request)?;
		let source_actor_internal = model::actor::Entity::ap_to_internal(&source_actor, self.db()).await?;
		let target_actor = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let usr = self.fetch_user(&target_actor).await?;
		let activity_model = model::activity::ActiveModel::new(&activity)?;
		model::activity::Entity::insert(activity_model)
			.exec(self.db()).await?;
		let internal_aid = model::activity::Entity::ap_to_internal(&aid, self.db()).await?;
		let relation_model = model::relation::ActiveModel {
			internal: NotSet,
			accept: Set(None),
			activity: Set(internal_aid),
			follower: Set(source_actor_internal),
			following: Set(usr.internal),
		};
		model::relation::Entity::insert(relation_model)
			.exec(self.db()).await?;
		let mut expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		if !expanded_addressing.contains(&target_actor) {
			expanded_addressing.push(target_actor);
		}
		self.address_to(Some(internal_aid), None, &expanded_addressing).await?;
		tracing::info!("{} wants to follow {}", source_actor, usr.id);
		Ok(())
	}

	async fn accept(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		// TODO what about TentativeAccept
		let aid = activity.id().ok_or_else(UpubError::bad_request)?.to_string();
		let target_actor = activity.actor().id().ok_or_else(UpubError::bad_request)?;
		let follow_request_id = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let follow_activity = model::activity::Entity::find_by_ap_id(&follow_request_id)
			.one(self.db())
			.await?
			.ok_or_else(UpubError::not_found)?;

		if follow_activity.object.unwrap_or("".into()) != target_actor {
			return Err(UpubError::forbidden());
		}

		let activity_model = model::activity::ActiveModel::new(&activity)?;
		model::activity::Entity::insert(activity_model)
			.exec(self.db())
			.await?;
		let accept_internal_id = model::activity::Entity::ap_to_internal(&aid, self.db()).await?;

		model::actor::Entity::update_many()
			.col_expr(
				model::actor::Column::FollowingCount,
				Expr::col(model::actor::Column::FollowingCount).add(1)
			)
			.filter(model::actor::Column::Id.eq(&follow_activity.actor))
			.exec(self.db())
			.await?;
		model::actor::Entity::update_many()
			.col_expr(
				model::actor::Column::FollowersCount,
				Expr::col(model::actor::Column::FollowersCount).add(1)
			)
			.filter(model::actor::Column::Id.eq(&follow_activity.actor))
			.exec(self.db())
			.await?;

		model::relation::Entity::update_many()
			.col_expr(model::relation::Column::Accept, Expr::value(Some(accept_internal_id)))
			.filter(model::relation::Column::Activity.eq(follow_activity.internal))
			.exec(self.db()).await?;

		tracing::info!("{} accepted follow request by {}", target_actor, follow_activity.actor);

		let mut expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		if !expanded_addressing.contains(&follow_activity.actor) {
			expanded_addressing.push(follow_activity.actor);
		}
		self.address_to(Some(accept_internal_id), None, &expanded_addressing).await?;
		Ok(())
	}

	async fn reject(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		// TODO what about TentativeReject?
		let aid = activity.id().ok_or_else(UpubError::bad_request)?.to_string();
		let uid = activity.actor().id().ok_or_else(UpubError::bad_request)?;
		let follow_request_id = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let follow_activity = model::activity::Entity::find_by_ap_id(&follow_request_id)
			.one(self.db())
			.await?
			.ok_or_else(UpubError::not_found)?;

		if follow_activity.object.unwrap_or("".into()) != uid {
			return Err(UpubError::forbidden());
		}

		let activity_model = model::activity::ActiveModel::new(&activity)?;
		model::activity::Entity::insert(activity_model)
			.exec(self.db())
			.await?;
		let internal_aid = model::activity::Entity::ap_to_internal(&aid, self.db()).await?;

		model::relation::Entity::delete_many()
			.filter(model::relation::Column::Activity.eq(internal_aid))
			.exec(self.db())
			.await?;

		tracing::info!("{} rejected follow request by {}", uid, follow_activity.actor);

		let mut expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		if !expanded_addressing.contains(&follow_activity.actor) {
			expanded_addressing.push(follow_activity.actor);
		}

		self.address_to(Some(internal_aid), None, &expanded_addressing).await?;
		Ok(())
	}

	async fn delete(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		model::actor::Entity::delete_by_ap_id(&oid).exec(self.db()).await.info_failed("failed deleting from users");
		model::object::Entity::delete_by_ap_id(&oid).exec(self.db()).await.info_failed("failed deleting from objects");
		tracing::debug!("deleted '{oid}'");
		Ok(())
	}

	async fn update(&self, _server: String, activity: serde_json::Value) -> crate::Result<()> {
		let uid = activity.actor().id().ok_or_else(UpubError::bad_request)?;
		let aid = activity.id().ok_or_else(UpubError::bad_request)?;
		let Some(object_node) = activity.object().extract() else {
			// TODO we could process non-embedded activities or arrays but im lazy rn
			tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&activity).unwrap());
			return Err(UpubError::unprocessable());
		};
		let oid = object_node.id().ok_or_else(UpubError::bad_request)?.to_string();

		let activity_model = model::activity::ActiveModel::new(&activity)?;
		model::activity::Entity::insert(activity_model)
			.exec(self.db())
			.await?;
		let internal_aid = model::activity::Entity::ap_to_internal(aid, self.db()).await?;

		let internal_oid = match object_node.object_type().ok_or_else(UpubError::bad_request)? {
			apb::ObjectType::Actor(_) => {
				let internal_uid = model::actor::Entity::ap_to_internal(&oid, self.db()).await?;
				let mut actor_model = model::actor::ActiveModel::new(&object_node)?;
				actor_model.internal = Set(internal_uid);
				actor_model.updated = Set(chrono::Utc::now());
				model::actor::Entity::update(actor_model)
					.exec(self.db())
					.await?;
				Some(internal_uid)
			},
			apb::ObjectType::Note => {
				let internal_oid = model::object::Entity::ap_to_internal(&oid, self.db()).await?;
				let mut object_model = model::object::ActiveModel::new(&object_node)?;
				object_model.internal = Set(internal_oid);
				object_model.updated = Set(chrono::Utc::now());
				model::object::Entity::update(object_model)
					.exec(self.db())
					.await?;
				Some(internal_oid)
			},
			t => {
				tracing::warn!("no side effects implemented for update type {t:?}");
				None
			},
		};

		tracing::info!("{} updated {}", uid, oid);
		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(Some(internal_aid), internal_oid, &expanded_addressing).await?;
		Ok(())
	}

	async fn undo(&self, server: String, activity: serde_json::Value) -> crate::Result<()> {
		let uid = activity.actor().id().ok_or_else(UpubError::bad_request)?;
		let internal_uid = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;
		// TODO in theory we could work with just object_id but right now only accept embedded
		let undone_activity = activity.object().extract().ok_or_else(UpubError::bad_request)?;
		let undone_activity_author = undone_activity.actor().id().ok_or_else(UpubError::bad_request)?;

		// can't undo activities from remote actors!
		if server != Context::server(&undone_activity_author) {
			return Err(UpubError::forbidden());
		};

		let activity_model = self.insert_activity(activity.clone(), Some(server)).await?;

		let targets = self.expand_addressing(activity.addressed()).await?;
		self.process_undo(internal_uid, activity).await?;

		self.address_to(Some(activity_model.internal), None, &targets).await?;

		Ok(())
	}
	
	async fn announce(&self, server: String, activity: serde_json::Value) -> crate::Result<()> {
		let uid = activity.actor().id().ok_or_else(|| UpubError::field("actor"))?;
		let actor = self.fetch_user(&uid).await?;
		let internal_uid = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;
		let announced_id = activity.object().id().ok_or_else(|| UpubError::field("object"))?;
		
		match self.pull(&announced_id).await? {
			PullResult::Actor(_) => Err(UpubError::unprocessable()),
			PullResult::Object(object) => {
				let object_model = self.resolve_object(object).await?;
				let activity_model = self.insert_activity(activity.clone(), Some(server.clone())).await?;

				// relays send us objects as Announce, but we don't really want to count those towards the
				// total shares count of an object, so just fetch the object and be done with it
				if !matches!(actor.actor_type, apb::ActorType::Person) {
					tracing::info!("relay {} broadcasted {}", activity_model.actor, announced_id);
					return Ok(())
				}

				let share = model::announce::ActiveModel {
					internal: NotSet,
					actor: Set(internal_uid),
					object: Set(object_model.internal),
					published: Set(activity.published().unwrap_or(chrono::Utc::now())),
				};

				let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
				self.address_to(Some(activity_model.internal), None, &expanded_addressing).await?;
				model::announce::Entity::insert(share)
					.exec(self.db()).await?;
				model::object::Entity::update_many()
					.col_expr(model::object::Column::Announces, Expr::col(model::object::Column::Announces).add(1))
					.filter(model::object::Column::Internal.eq(object_model.internal))
					.exec(self.db())
					.await?;

				tracing::info!("{} shared {}", activity_model.actor, announced_id);
				Ok(())
			},
			PullResult::Activity(activity) => {
				// groups update all members of other things that happen inside, process those
				let server = Context::server(activity.id().unwrap_or_default());
				match activity.activity_type().ok_or_else(UpubError::bad_request)? {
					apb::ActivityType::Like | apb::ActivityType::EmojiReact => Ok(self.like(server, activity).await?),
					apb::ActivityType::Create => Ok(self.create(server, activity).await?),
					apb::ActivityType::Undo => Ok(self.undo(server, activity).await?),
					apb::ActivityType::Delete => Ok(self.delete(server, activity).await?),
					apb::ActivityType::Update => Ok(self.update(server, activity).await?),
					x => {
						tracing::warn!("ignoring unhandled announced activity of type {x:?}");
						Err(StatusCode::NOT_IMPLEMENTED.into())
					},
				}
			},
		}
	}
}
