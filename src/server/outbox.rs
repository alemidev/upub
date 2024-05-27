use apb::{target::Addressed, Activity, ActivityMut, Base, BaseMut, Node, Object, ObjectMut};
use reqwest::StatusCode;
use sea_orm::{sea_query::Expr, ActiveValue::{Set, NotSet, Unchanged}, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns};

use crate::{errors::UpubError, model};

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
				.set_content(content.as_deref())
				.set_url(Node::maybe_link(self.cfg().instance.frontend.as_ref().map(|x| format!("{x}/objects/{raw_oid}")))),
			Some(self.domain().to_string()),
		).await?;

		let activity_model = model::activity::ActiveModel {
			internal: NotSet,
			id: Set(aid.clone()),
			activity_type: Set(apb::ActivityType::Create),
			actor: Set(uid.clone()),
			object: Set(Some(oid.clone())),
			target: Set(None),
			cc: Set(object_model.cc.clone()),
			bcc: Set(object_model.bcc.clone()),
			to: Set(object_model.to.clone()),
			bto: Set(object_model.bto.clone()),
			published: Set(object_model.published),
		};

		model::activity::Entity::insert(activity_model).exec(self.db()).await?;

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
				.set_to(activity.to())
				.set_bto(activity.bto())
				.set_cc(activity.cc())
				.set_bcc(activity.bcc())
				.set_url(Node::maybe_link(self.cfg().instance.frontend.as_ref().map(|x| format!("{x}/objects/{raw_oid}")))),
			Some(self.domain().to_string()),
		).await?;

		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
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
		let obj_model = self.fetch_object(&oid).await?;
		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;

		let internal_uid = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;

		let like_model = model::like::ActiveModel {
			internal: NotSet,
			actor: Set(internal_uid),
			object: Set(obj_model.internal),
			published: Set(chrono::Utc::now()),
		};

		model::like::Entity::insert(like_model).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model)
			.exec(self.db()).await?;
		model::object::Entity::update_many()
			.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
			.filter(model::object::Column::Internal.eq(obj_model.internal))
			.exec(self.db())
			.await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn follow(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let target = activity.object().id().ok_or_else(UpubError::bad_request)?;

		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;

		let follower_internal = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;
		let following_internal = model::actor::Entity::ap_to_internal(&target, self.db()).await?;

		model::activity::Entity::insert(activity_model)
			.exec(self.db()).await?;

		let internal_aid = model::activity::Entity::ap_to_internal(&aid, self.db()).await?;

		let relation_model = model::relation::ActiveModel {
			internal: NotSet,
			follower: Set(follower_internal),
			following: Set(following_internal),
			activity: Set(internal_aid),
			accept: Set(None),
		};

		model::relation::Entity::insert(relation_model)
			.exec(self.db()).await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn accept(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let accepted_id = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let accepted_activity = model::activity::Entity::find_by_ap_id(&accepted_id)
			.one(self.db()).await?
			.ok_or_else(UpubError::not_found)?;

		if accepted_activity.activity_type != apb::ActivityType::Follow {
			return Err(UpubError::bad_request());
		}
		if uid != accepted_activity.object.ok_or_else(UpubError::bad_request)? {
			return Err(UpubError::forbidden());
		}

		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;

		let internal_aid = model::activity::Entity::ap_to_internal(&aid, self.db()).await?;

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
				model::relation::Entity::update_many()
					.filter(model::relation::Column::Activity.eq(accepted_activity.internal))
					.col_expr(model::relation::Column::Accept, Expr::value(Some(internal_aid)))
					.exec(self.db()).await?;
			},
			t => tracing::error!("no side effects implemented for accepting {t:?}"),
		}

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn reject(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let rejected_id = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let rejected_activity = model::activity::Entity::find_by_ap_id(&rejected_id)
			.one(self.db()).await?
			.ok_or_else(UpubError::not_found)?;

		if rejected_activity.activity_type != apb::ActivityType::Follow {
			return Err(UpubError::bad_request());
		}
		if uid != rejected_activity.object.ok_or_else(UpubError::bad_request)? {
			return Err(UpubError::forbidden());
		}

		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;
		model::activity::Entity::insert(activity_model)
			.exec(self.db()).await?;

		let internal_aid = model::activity::Entity::ap_to_internal(&aid, self.db()).await?;

		model::relation::Entity::delete_many()
			.filter(model::relation::Column::Activity.eq(internal_aid))
			.exec(self.db())
			.await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn undo(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let old_aid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let old_activity = model::activity::Entity::find_by_ap_id(&old_aid)
			.one(self.db())
			.await?
			.ok_or_else(UpubError::not_found)?;
		if old_activity.actor != uid {
			return Err(UpubError::forbidden());
		}
		let activity_object = old_activity.object.ok_or_else(UpubError::bad_request)?;
		let actor_internal = model::actor::Entity::ap_to_internal(&old_activity.actor, self.db()).await?;

		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db())
			.await?;

		match old_activity.activity_type {
			apb::ActivityType::Like => {
				let object_internal = model::object::Entity::ap_to_internal(&activity_object, self.db()).await?;
				model::like::Entity::delete_many()
					.filter(model::like::Column::Actor.eq(actor_internal))
					.filter(model::like::Column::Object.eq(object_internal))
					.exec(self.db())
					.await?;
			},
			apb::ActivityType::Follow => {
				let target_internal = model::actor::Entity::ap_to_internal(&activity_object, self.db()).await?;
				model::relation::Entity::delete_many()
					.filter(model::relation::Column::Follower.eq(actor_internal))
					.filter(model::relation::Column::Following.eq(target_internal))
					.exec(self.db())
					.await?;
			},
			t => tracing::error!("extra side effects for activity {t:?} not implemented"),
		}

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}

	async fn delete(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;

		let object = model::object::Entity::find_by_ap_id(&oid)
			.one(self.db())
			.await?
			.ok_or_else(UpubError::not_found)?;

		if uid != object.attributed_to.ok_or_else(UpubError::forbidden)? {
			// can't change objects of others, and objects from noone count as others
			return Err(UpubError::forbidden());
		}

		let addressed = activity.addressed();
		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;

		model::activity::Entity::insert(activity_model)
			.exec(self.db())
			.await?;

		model::object::Entity::delete_by_ap_id(&oid)
			.exec(self.db())
			.await?;

		self.dispatch(&uid, addressed, &aid, None).await?;

		Ok(aid)
	}

	async fn update(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let object_node = activity.object().extract().ok_or_else(UpubError::bad_request)?;
		let addressed = activity.addressed();
		let target = object_node.id().ok_or_else(UpubError::bad_request)?.to_string();

		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;

		model::activity::Entity::insert(activity_model)
			.exec(self.db()).await?;

		match object_node.object_type() {
			Some(apb::ObjectType::Actor(_)) => {
				let old_actor_model = model::actor::Entity::find_by_ap_id(&target)
					.one(self.db())
					.await?
					.ok_or_else(UpubError::not_found)?;

				if old_actor_model.id != uid {
					// can't change user fields of others
					return Err(UpubError::forbidden());
				}

				let mut new_actor_model = model::actor::ActiveModel {
					internal: Unchanged(old_actor_model.internal),
					..Default::default()
				};

				if let Some(name) = object_node.name() {
					new_actor_model.name = Set(Some(name.to_string()));
				}
				if let Some(summary) = object_node.summary() {
					new_actor_model.summary = Set(Some(summary.to_string()));
				}
				if let Some(image) = object_node.image().id() {
					new_actor_model.image = Set(Some(image));
				}
				if let Some(icon) = object_node.icon().id() {
					new_actor_model.icon = Set(Some(icon));
				}
				new_actor_model.updated = Set(chrono::Utc::now());

				model::actor::Entity::update(new_actor_model)
					.exec(self.db()).await?;
			},
			Some(apb::ObjectType::Note) => {
				let old_object_model = model::object::Entity::find_by_ap_id(&target)
					.one(self.db())
					.await?
					.ok_or_else(UpubError::not_found)?;

				if uid != old_object_model.attributed_to.ok_or_else(UpubError::forbidden)? {
					// can't change objects of others
					return Err(UpubError::forbidden());
				}

				let mut new_object_model = model::object::ActiveModel {
					internal: Unchanged(old_object_model.internal),
					..Default::default()
				};

				if let Some(name) = object_node.name() {
					new_object_model.name = Set(Some(name.to_string()));
				}
				if let Some(summary) = object_node.summary() {
					new_object_model.summary = Set(Some(summary.to_string()));
				}
				if let Some(content) = object_node.content() {
					new_object_model.content = Set(Some(content.to_string()));
				}
				new_object_model.updated = Set(chrono::Utc::now());

				model::object::Entity::update(new_object_model)
					.exec(self.db()).await?;
			},
			_ => return Err(UpubError::Status(StatusCode::NOT_IMPLEMENTED)),
		}

		self.dispatch(&uid, addressed, &aid, None).await?;

		Ok(aid)
	}

	async fn announce(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let obj = self.fetch_object(&oid).await?;
		let internal_uid = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;

		let activity_model = model::activity::ActiveModel::new(
			&activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
		)?;

		let share_model = model::announce::ActiveModel {
			internal: NotSet,
			actor: Set(internal_uid),
			object: Set(obj.internal),
			published: Set(chrono::Utc::now()),
		};
		model::activity::Entity::insert(activity_model)
			.exec(self.db()).await?;
		model::announce::Entity::insert(share_model).exec(self.db()).await?;
		model::object::Entity::update_many()
			.col_expr(model::object::Column::Announces, Expr::col(model::object::Column::Announces).add(1))
			.filter(model::object::Column::Internal.eq(obj.internal))
			.exec(self.db())
			.await?;

		self.dispatch(&uid, activity_targets, &aid, None).await?;

		Ok(aid)
	}
}
