use apb::{target::Addressed, Activity, ActivityMut, ActorMut, BaseMut, Node, Object, ObjectMut, PublicKeyMut};
use reqwest::StatusCode;
use sea_orm::{sea_query::Expr, ActiveModelTrait, ActiveValue::{Set, NotSet}, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns};

use crate::{errors::UpubError, model, routes::activitypub::jsonld::LD};

use super::{fetcher::Fetcher, normalizer::Normalizer, Context};


#[axum::async_trait]
impl apb::server::Outbox for Context {
	type Error = UpubError;
	type Object = serde_json::Value;
	type Activity = serde_json::Value;

	async fn create_note(&self, uid: String, object: serde_json::Value) -> crate::Result<String> {
		// TODO regex hell, here i come...
		let re = regex::Regex::new(r"@(.+)@([^ ]+)").expect("failed compiling regex pattern");
		let raw_oid = uuid::Uuid::new_v4().to_string();
		let oid = self.oid(&raw_oid);
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = object.addressed();

		let mut content = object.content().map(|x| x.to_string());
		if let Some(c) = content {
			let mut tmp = mdhtml::safe_markdown(&c);
			for (full, [user, domain]) in re.captures_iter(&tmp.clone()).map(|x| x.extract()) {
				if let Ok(Some(uid)) = model::actor::Entity::find()
					.filter(model::actor::Column::PreferredUsername.eq(user))
					.filter(model::actor::Column::Domain.eq(domain))
					.select_only()
					.select_column(model::actor::Column::Id)
					.into_tuple::<String>()
					.one(self.db())
					.await
				{
					tmp = tmp.replacen(full, &format!("<a href=\"{uid}\" class=\"u-url mention\">@{user}</a>"), 1);
				}
			}
			content = Some(tmp);
		}

		let object_model = self.insert_object(
			object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
				.set_content(content.as_deref())
				.set_url(Node::maybe_link(self.cfg().instance.frontend.as_ref().map(|x| format!("{x}/objects/{raw_oid}")))),
			Some(self.domain().to_string()),
		).await?;

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

		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;

		Ok(aid)
	}

	async fn create(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let Some(object) = activity.object().extract() else {
			return Err(UpubError::bad_request());
		};

		let raw_oid = uuid::Uuid::new_v4().to_string();
		let oid = self.oid(&raw_oid);
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();

		self.insert_object(
			object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
				.set_to(activity.to())
				.set_bto(activity.bto())
				.set_cc(activity.cc())
				.set_bcc(activity.bcc()),
			Some(self.domain().to_string()),
		).await?;

		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
				.set_object(Node::link(oid.clone()))
		)?;

		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;
		Ok(aid)
	}

	async fn like(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
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
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
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
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
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
				model::actor::Entity::update_many()
					.col_expr(
						model::actor::Column::FollowersCount,
						Expr::col(model::actor::Column::FollowersCount).add(1)
					)
					.filter(model::actor::Column::Id.eq(&uid))
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
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
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
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
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
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let object_node = activity.object().extract().ok_or_else(UpubError::bad_request)?;

		match object_node.object_type() {
			Some(apb::ObjectType::Actor(_)) => {
				let mut actor_model = model::actor::Model::new(
					&object_node
						// TODO must set these, but we will ignore them
						.set_actor_type(Some(apb::ActorType::Person))
						.set_public_key(apb::Node::object(
							serde_json::Value::new_object().set_public_key_pem("")
						))
				)?;
				let old_actor_model = model::actor::Entity::find_by_id(&actor_model.id)
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
				update_model.reset(model::actor::Column::Name);
				update_model.reset(model::actor::Column::Summary);
				update_model.reset(model::actor::Column::Image);
				update_model.reset(model::actor::Column::Icon);

				model::actor::Entity::update(update_model)
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
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		self.fetch_object(&oid).await?;
		let activity_model = model::activity::Model::new(
			&activity
				.set_id(Some(&aid))
				.set_published(Some(chrono::Utc::now()))
				.set_actor(Node::link(uid.clone()))
		)?;

		let share_model = model::announce::ActiveModel {
			internal: NotSet,
			actor: Set(uid.clone()),
			object: Set(oid.clone()),
			published: Set(chrono::Utc::now()),
		};
		model::announce::Entity::insert(share_model).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;
		model::object::Entity::update_many()
			.col_expr(model::object::Column::Announces, Expr::col(model::object::Column::Announces).add(1))
			.filter(model::object::Column::Id.eq(oid))
			.exec(self.db())
			.await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}
}
