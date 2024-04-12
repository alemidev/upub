use apb::{target::Addressed, Activity, Base, Object};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

use crate::{errors::{LoggableError, UpubError}, model};

use super::Context;


#[axum::async_trait]
impl apb::server::Inbox for Context {
	type Error = UpubError;
	type Activity = serde_json::Value;

	async fn create(&self, activity: serde_json::Value) -> crate::Result<()> {
		let activity_model = model::activity::Model::new(&activity)?;
		let Some(object_node) = activity.object().extract() else {
			// TODO we could process non-embedded activities or arrays but im lazy rn
			tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&activity).unwrap());
			return Err(UpubError::unprocessable());
		};
		let object_model = model::object::Model::new(&object_node)?;
		let aid = activity_model.id.clone();
		let oid = object_model.id.clone();
		model::object::Entity::insert(object_model.into_active_model()).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model()).exec(self.db()).await?;
		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(&aid, Some(&oid), &expanded_addressing).await?;
		tracing::info!("{} posted {}", aid, oid);
		Ok(())
	}

	async fn like(&self, activity: serde_json::Value) -> crate::Result<()> {
		let aid = activity.actor().id().ok_or(UpubError::bad_request())?;
		let oid = activity.object().id().ok_or(UpubError::bad_request())?;
		let like = model::like::ActiveModel {
			id: sea_orm::ActiveValue::NotSet,
			actor: sea_orm::Set(aid.clone()),
			likes: sea_orm::Set(oid.clone()),
			date: sea_orm::Set(chrono::Utc::now()),
		};
		match model::like::Entity::insert(like).exec(self.db()).await {
			Err(sea_orm::DbErr::RecordNotInserted) => Err(UpubError::not_modified()),
			Err(sea_orm::DbErr::Exec(_)) => Err(UpubError::not_modified()), // bad fix for sqlite
			Err(e) => {
				tracing::error!("unexpected error procesing like from {aid} to {oid}: {e}");
				Err(UpubError::internal_server_error())
			}
			Ok(_) => {
				let activity_model = model::activity::Model::new(&activity)?.into_active_model();
				model::activity::Entity::insert(activity_model)
					.exec(self.db())
					.await?;
				let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
				self.address_to(&aid, None, &expanded_addressing).await?;
				model::object::Entity::update_many()
					.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
					.filter(model::object::Column::Id.eq(oid.clone()))
					.exec(self.db())
					.await?;
				tracing::info!("{} liked {}", aid, oid);
				Ok(())
			},
		}
	}

	async fn follow(&self, activity: serde_json::Value) -> crate::Result<()> {
		let activity_model = model::activity::Model::new(&activity)?;
		let aid = activity_model.id.clone();
		tracing::info!("{} wants to follow {}", activity_model.actor, activity_model.object.as_deref().unwrap_or("<no-one???>"));
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;
		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(&aid, None, &expanded_addressing).await?;
		Ok(())
	}

	async fn accept(&self, activity: serde_json::Value) -> crate::Result<()> {
		// TODO what about TentativeAccept
		let activity_model = model::activity::Model::new(&activity)?;
		let Some(follow_request_id) = &activity_model.object else {
			return Err(UpubError::bad_request());
		};
		let Some(follow_activity) = model::activity::Entity::find_by_id(follow_request_id)
			.one(self.db()).await?
		else {
			return Err(UpubError::not_found());
		};
		if follow_activity.object.unwrap_or("".into()) != activity_model.actor {
			return Err(UpubError::forbidden());
		}

		tracing::info!("{} accepted follow request by {}", activity_model.actor, follow_activity.actor);

		model::activity::Entity::insert(activity_model.clone().into_active_model())
			.exec(self.db())
			.await?;
		model::relation::Entity::insert(
			model::relation::ActiveModel {
				follower: Set(follow_activity.actor),
				following: Set(activity_model.actor),
				..Default::default()
			}
		).exec(self.db()).await?;

		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(&activity_model.id, None, &expanded_addressing).await?;
		Ok(())
	}

	async fn reject(&self, activity: serde_json::Value) -> crate::Result<()> {
		// TODO what about TentativeReject?
		let activity_model = model::activity::Model::new(&activity)?;
		let Some(follow_request_id) = &activity_model.object else {
			return Err(UpubError::bad_request());
		};
		let Some(follow_activity) = model::activity::Entity::find_by_id(follow_request_id)
			.one(self.db()).await?
		else {
			return Err(UpubError::not_found());
		};
		if follow_activity.object.unwrap_or("".into()) != activity_model.actor {
			return Err(UpubError::forbidden());
		}

		tracing::info!("{} rejected follow request by {}", activity_model.actor, follow_activity.actor);

		model::activity::Entity::insert(activity_model.clone().into_active_model())
			.exec(self.db())
			.await?;

		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(&activity_model.id, None, &expanded_addressing).await?;
		Ok(())
	}

	async fn delete(&self, activity: serde_json::Value) -> crate::Result<()> {
		// TODO verify the signature before just deleting lmao
		let oid = activity.object().id().ok_or(UpubError::bad_request())?;
		// TODO maybe we should keep the tombstone?
		model::user::Entity::delete_by_id(&oid).exec(self.db()).await.info_failed("failed deleting from users");
		model::activity::Entity::delete_by_id(&oid).exec(self.db()).await.info_failed("failed deleting from activities");
		model::object::Entity::delete_by_id(&oid).exec(self.db()).await.info_failed("failed deleting from objects");
		Ok(())
	}

	async fn update(&self, activity: serde_json::Value) -> crate::Result<()> {
		let activity_model = model::activity::Model::new(&activity)?;
		let Some(object_node) = activity.object().extract() else {
			// TODO we could process non-embedded activities or arrays but im lazy rn
			tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&activity).unwrap());
			return Err(UpubError::unprocessable());
		};
		let aid = activity_model.id.clone();
		let Some(oid) = object_node.id().map(|x| x.to_string()) else {
			return Err(UpubError::bad_request());
		};
		match object_node.object_type() {
			Some(apb::ObjectType::Actor(_)) => {
				// TODO oof here is an example of the weakness of this model, we have to go all the way
				// back up to serde_json::Value because impl Object != impl Actor
				let actor_model = model::user::Model::new(&object_node)?;
				model::user::Entity::update(actor_model.into_active_model())
					.exec(self.db()).await?;
			},
			Some(apb::ObjectType::Note) => {
				let object_model = model::object::Model::new(&object_node)?;
				model::object::Entity::update(object_model.into_active_model())
					.exec(self.db()).await?;
			},
			Some(t) => tracing::warn!("no side effects implemented for update type {t:?}"),
			None => tracing::warn!("empty type on embedded updated object"),
		}

		tracing::info!("{} updated {}", aid, oid);
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db())
			.await?;
		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(&aid, Some(&oid), &expanded_addressing).await?;
		Ok(())
	}

	async fn undo(&self, _activity: serde_json::Value) -> crate::Result<()> {
		todo!()
	}
}