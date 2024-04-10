use apb::{target::Addressed, Activity, ActivityMut, BaseMut, Node, ObjectMut};
use sea_orm::{EntityTrait, IntoActiveModel, Set};

use crate::{errors::UpubError, model};

use super::Context;


#[axum::async_trait]
impl apb::server::Outbox for Context {
	type Error = UpubError;
	type Object = serde_json::Value;
	type Activity = serde_json::Value;

	async fn create_note(&self, uid: String, object: serde_json::Value) -> crate::Result<String> {
		let oid = self.oid(uuid::Uuid::new_v4().to_string());
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = object.addressed();
		let object_model = model::object::Model::new(
			&object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		let activity_model = model::activity::Model {
			id: aid.clone(),
			activity_type: apb::ActivityType::Create,
			actor: uid.clone(),
			object: Some(oid.clone()),
			target: None,
			cc: object_model.cc.clone(),
			bcc: object_model.bcc.clone(),
			to: object_model.to.clone(),
			bto: object_model.bto.clone(),
			published: object_model.published,
		};

		model::object::Entity::insert(object_model.into_active_model())
			.exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;

		Ok(aid)
	}

	async fn create(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let Some(object) = activity.object().extract() else {
			return Err(UpubError::bad_request());
		};

		let oid = self.oid(uuid::Uuid::new_v4().to_string());
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let mut object_model = model::object::Model::new(
			&object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		let mut activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		object_model.to = activity_model.to.clone();
		object_model.bto = activity_model.bto.clone();
		object_model.cc = activity_model.cc.clone();
		object_model.bcc = activity_model.bcc.clone();
		activity_model.object = Some(oid.clone());

		model::object::Entity::insert(object_model.into_active_model())
			.exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;

		Ok(aid)
	}
		

	async fn like(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let Some(oid) = activity.object().id() else {
			return Err(UpubError::bad_request());
		};
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_published(Some(chrono::Utc::now()))
				.set_actor(Node::link(uid.clone()))
		)?;

		let like_model = model::like::ActiveModel {
			actor: Set(uid.clone()),
			likes: Set(oid),
			date: Set(chrono::Utc::now()),
			..Default::default()
		};
		model::like::Entity::insert(like_model).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn follow(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		if activity.object().id().is_none() {
			return Err(UpubError::bad_request());
		}

		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn accept(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		if activity.object().id().is_none() {
			return Err(UpubError::bad_request());
		}
		let Some(accepted_id) = activity.object().id() else {
			return Err(UpubError::bad_request());
		};
		let Some(accepted_activity) = model::activity::Entity::find_by_id(accepted_id)
			.one(self.db()).await?
		else {
			return Err(UpubError::not_found());
		};

		match accepted_activity.activity_type {
			apb::ActivityType::Follow => {
				model::relation::Entity::insert(
					model::relation::ActiveModel {
						follower: Set(accepted_activity.actor), following: Set(uid.clone()),
						..Default::default()
					}
				).exec(self.db()).await?;
			},
			t => tracing::warn!("no side effects implemented for accepting {t:?}"),
		}

		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn reject(&self, _uid: String, _activity: serde_json::Value) -> crate::Result<String> {
		todo!()
	}

	async fn undo(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		{
			let Some(old_aid) = activity.object().id() else {
				return Err(UpubError::bad_request());
			};
			let Some(old_activity) = model::activity::Entity::find_by_id(old_aid)
				.one(self.db()).await?
			else {
				return Err(UpubError::not_found());
			};
			if old_activity.actor != uid {
				return Err(UpubError::forbidden());
			}
			match old_activity.activity_type {
				apb::ActivityType::Like => {
					model::like::Entity::delete(model::like::ActiveModel {
						actor: Set(old_activity.actor), likes: Set(old_activity.object.unwrap_or("".into())),
						..Default::default()
					}).exec(self.db()).await?;
				},
				apb::ActivityType::Follow => {
					model::relation::Entity::delete(model::relation::ActiveModel {
						follower: Set(old_activity.actor), following: Set(old_activity.object.unwrap_or("".into())),
						..Default::default()
					}).exec(self.db()).await?;
				},
				t => tracing::warn!("extra side effects for activity {t:?} not implemented"),
			}
		}
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db())
			.await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}
}
