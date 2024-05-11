use apb::{target::Addressed, Activity, ActivityMut, ActorMut, BaseMut, Node, Object, ObjectMut, PublicKeyMut};
use reqwest::StatusCode;
use sea_orm::{sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

use crate::{errors::UpubError, model, routes::activitypub::jsonld::LD};

use super::{fetcher::Fetcher, Context};


#[axum::async_trait]
impl apb::server::Outbox for Context {
	type Error = UpubError;
	type Object = serde_json::Value;
	type Activity = serde_json::Value;

	async fn create_note(&self, uid: String, object: serde_json::Value) -> crate::Result<String> {
		let raw_oid = uuid::Uuid::new_v4().to_string();
		let oid = self.oid(raw_oid.clone());
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = object.addressed();
		let mut object_model = model::object::Model::new(
			&object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;
		if let Some(content) = object_model.content {
			object_model.content = Some(mdhtml::safe_markdown(&content));
		}
		match (&object_model.in_reply_to, &object_model.context) {
			(Some(reply_id), None) => // get context from replied object
				object_model.context = self.fetch_object(reply_id).await?.context,
			(None, None) => // generate a new context
				object_model.context = Some(crate::url!(self, "/context/{}", uuid::Uuid::new_v4().to_string())),
			(_, Some(_)) => {}, // leave it as set by user
		}
		let reply_to = object_model.in_reply_to.clone();

		if let Some(fe_url) = &self.cfg().instance.frontend {
			object_model.url = Some(format!("{fe_url}/objects/{raw_oid}")); 
		}

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
		model::user::Entity::update_many()
			.col_expr(model::user::Column::StatusesCount, Expr::col(model::user::Column::StatusesCount).add(1))
			.filter(model::user::Column::Id.eq(&uid))
			.exec(self.db())
			.await?;
		if let Some(reply_to) = reply_to {
			model::object::Entity::update_many()
				.filter(model::object::Column::Id.eq(reply_to))
				.col_expr(model::object::Column::Comments, Expr::col(model::object::Column::Comments).add(1))
				.exec(self.db())
				.await?;
		}

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;

		Ok(aid)
	}

	async fn create(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let Some(object) = activity.object().extract() else {
			return Err(UpubError::bad_request());
		};

		let raw_oid = uuid::Uuid::new_v4().to_string();
		let oid = self.oid(raw_oid.clone());
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
		activity_model.object = Some(oid.clone());
		object_model.to = activity_model.to.clone();
		object_model.bto = activity_model.bto.clone();
		object_model.cc = activity_model.cc.clone();
		object_model.bcc = activity_model.bcc.clone();
		if let Some(content) = object_model.content {
			object_model.content = Some(mdhtml::safe_markdown(&content));
		}
		match (&object_model.in_reply_to, &object_model.context) {
			(Some(reply_id), None) => // get context from replied object
				object_model.context = self.fetch_object(reply_id).await?.context,
			(None, None) => // generate a new context
				object_model.context = Some(crate::url!(self, "/context/{}", uuid::Uuid::new_v4().to_string())),
			(_, Some(_)) => {}, // leave it as set by user
		}
		let reply_to = object_model.in_reply_to.clone();
		if let Some(fe_url) = &self.cfg().instance.frontend {
			object_model.url = Some(format!("{fe_url}/objects/{raw_oid}")); 
		}

		model::object::Entity::insert(object_model.into_active_model())
			.exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;
		model::user::Entity::update_many()
			.col_expr(model::user::Column::StatusesCount, Expr::col(model::user::Column::StatusesCount).add(1))
			.filter(model::user::Column::Id.eq(&uid))
			.exec(self.db())
			.await?;
		if let Some(reply_to) = reply_to {
			model::object::Entity::update_many()
				.filter(model::object::Column::Id.eq(reply_to))
				.col_expr(model::object::Column::Comments, Expr::col(model::object::Column::Comments).add(1))
				.exec(self.db())
				.await?;
		}

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;

		Ok(aid)
	}
		

	async fn like(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		self.fetch_object(&oid).await?;
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_published(Some(chrono::Utc::now()))
				.set_actor(Node::link(uid.clone()))
		)?;

		let like_model = model::like::ActiveModel {
			actor: Set(uid.clone()),
			likes: Set(oid.clone()),
			date: Set(chrono::Utc::now()),
			..Default::default()
		};
		model::like::Entity::insert(like_model).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;
		model::object::Entity::update_many()
			.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
			.filter(model::object::Column::Id.eq(oid))
			.exec(self.db())
			.await?;

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
				model::user::Entity::update_many()
					.col_expr(
						model::user::Column::FollowersCount,
						Expr::col(model::user::Column::FollowersCount).add(1)
					)
					.filter(model::user::Column::Id.eq(&uid))
					.exec(self.db())
					.await?;
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
		let old_aid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let old_activity = model::activity::Entity::find_by_id(old_aid)
			.one(self.db())
			.await?
			.ok_or_else(UpubError::not_found)?;
		if old_activity.actor != uid {
			return Err(UpubError::forbidden());
		}
		match old_activity.activity_type {
			apb::ActivityType::Like => {
				model::like::Entity::delete_many()
					.filter(model::like::Column::Actor.eq(old_activity.actor))
					.filter(model::like::Column::Likes.eq(old_activity.object.unwrap_or("".into())))
					.exec(self.db())
					.await?;
			},
			apb::ActivityType::Follow => {
				model::relation::Entity::delete_many()
					.filter(model::relation::Column::Follower.eq(old_activity.actor))
					.filter(model::relation::Column::Following.eq(old_activity.object.unwrap_or("".into())))
					.exec(self.db())
					.await?;
			},
			t => tracing::warn!("extra side effects for activity {t:?} not implemented"),
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

	async fn delete(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;

		let object = model::object::Entity::find_by_id(&oid)
			.one(self.db())
			.await?
			.ok_or_else(UpubError::not_found)?;

		let Some(author_id) = object.attributed_to else {
			// can't change local objects attributed to nobody
			return Err(UpubError::forbidden())
		};

		if author_id != uid {
			// can't change objects of others
			return Err(UpubError::forbidden());
		}

		let addressed = activity.addressed();
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;

		model::object::Entity::delete_by_id(&oid)
			.exec(self.db())
			.await?;

		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db())
			.await?;

		self.dispatch(&uid, addressed, &aid, None).await?;

		Ok(aid)
	}

	async fn update(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let object_node = activity.object().extract().ok_or_else(UpubError::bad_request)?;

		match object_node.object_type() {
			Some(apb::ObjectType::Actor(_)) => {
				let mut actor_model = model::user::Model::new(
					&object_node
						// TODO must set these, but we will ignore them
						.set_actor_type(Some(apb::ActorType::Person))
						.set_public_key(apb::Node::object(
							serde_json::Value::new_object().set_public_key_pem("")
						))
				)?;
				let old_actor_model = model::user::Entity::find_by_id(&actor_model.id)
					.one(self.db())
					.await?
					.ok_or_else(UpubError::not_found)?;

				if old_actor_model.id != uid {
					// can't change user fields of others
					return Err(UpubError::forbidden());
				}

				if actor_model.name.is_none() { actor_model.name = old_actor_model.name }
				if actor_model.summary.is_none() { actor_model.summary = old_actor_model.summary }
				if actor_model.image.is_none() { actor_model.image = old_actor_model.image }
				if actor_model.icon.is_none() { actor_model.icon = old_actor_model.icon }

				let mut update_model = actor_model.into_active_model();
				update_model.updated = sea_orm::Set(chrono::Utc::now());
				update_model.reset(model::user::Column::Name);
				update_model.reset(model::user::Column::Summary);
				update_model.reset(model::user::Column::Image);
				update_model.reset(model::user::Column::Icon);

				model::user::Entity::update(update_model)
					.exec(self.db()).await?;
			},
			Some(apb::ObjectType::Note) => {
				let mut object_model = model::object::Model::new(
					&object_node.set_published(Some(chrono::Utc::now()))
				)?;

				let old_object_model = model::object::Entity::find_by_id(&object_model.id)
					.one(self.db())
					.await?
					.ok_or_else(UpubError::not_found)?;

				// can't change local objects attributed to nobody
				let author_id = old_object_model.attributed_to.ok_or_else(UpubError::forbidden)?;
				if author_id != uid {
					// can't change objects of others
					return Err(UpubError::forbidden());
				}

				if object_model.name.is_none() { object_model.name = old_object_model.name }
				if object_model.summary.is_none() { object_model.summary = old_object_model.summary }
				if object_model.content.is_none() { object_model.content = old_object_model.content }

				let mut update_model = object_model.into_active_model();
				update_model.updated = sea_orm::Set(Some(chrono::Utc::now()));
				update_model.reset(model::object::Column::Name);
				update_model.reset(model::object::Column::Summary);
				update_model.reset(model::object::Column::Content);
				update_model.reset(model::object::Column::Sensitive);

				model::object::Entity::update(update_model)
					.exec(self.db()).await?;
			},
			_ => return Err(UpubError::Status(StatusCode::NOT_IMPLEMENTED)),
		}

		let addressed = activity.addressed();
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
		)?;

		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, addressed, &aid, None).await?;

		Ok(aid)
	}

	async fn announce(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		self.fetch_object(&oid).await?;
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_published(Some(chrono::Utc::now()))
				.set_actor(Node::link(uid.clone()))
		)?;

		let share_model = model::share::ActiveModel {
			actor: Set(uid.clone()),
			shares: Set(oid.clone()),
			date: Set(chrono::Utc::now()),
			..Default::default()
		};
		model::share::Entity::insert(share_model).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;
		model::object::Entity::update_many()
			.col_expr(model::object::Column::Shares, Expr::col(model::object::Column::Shares).add(1))
			.filter(model::object::Column::Id.eq(oid))
			.exec(self.db())
			.await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}
}
