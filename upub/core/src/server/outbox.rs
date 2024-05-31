use apb::{target::Addressed, Activity, ActivityMut, Base, BaseMut, Node, Object, ObjectMut};
use reqwest::StatusCode;
use sea_orm::{sea_query::Expr, ActiveValue::{Set, NotSet, Unchanged}, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns};

use crate::{errors::UpubError, model, ext::AnyQuery};

use super::{addresser::Addresser, fetcher::Fetcher, normalizer::Normalizer, side_effects::SideEffects, Context};


#[axum::async_trait]
impl apb::server::Outbox for Context {
	type Error = UpubError;
	type Object = serde_json::Value;
	type Activity = serde_json::Value;

	async fn create_note(&self, uid: String, object: serde_json::Value) -> crate::Result<String> {
		self.create(
			uid,
			apb::new()
				.set_activity_type(Some(apb::ActivityType::Create))
				.set_to(object.to())
				.set_bto(object.bto())
				.set_cc(object.cc())
				.set_bcc(object.bcc())
				.set_object(Node::object(object))
		).await
	}

	async fn create(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let Some(object) = activity.object().extract() else {
			return Err(UpubError::bad_request());
		};

		let raw_oid = uuid::Uuid::new_v4().to_string();
		let oid = self.oid(&raw_oid);
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();

		if let Some(reply) = object.in_reply_to().id() {
			self.fetch_object(&reply).await?;
		}

		// TODO regex hell here i come...
		let re = regex::Regex::new(r"@(.+)@([^ ]+)").expect("failed compiling regex pattern");
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

		self.insert_object(
			object
				.set_id(Some(&oid))
				.set_attributed_to(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now()))
				.set_content(content.as_deref())
				.set_url(Node::maybe_link(self.cfg().instance.frontend.as_ref().map(|x| format!("{x}/objects/{raw_oid}")))),
			Some(self.domain().to_string()),
		).await?;

		self.insert_activity(
			activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_object(Node::link(oid.clone()))
				.set_published(Some(chrono::Utc::now())),
			Some(self.domain().to_string()),
		).await?;

		self.dispatch(&uid, activity_targets, &aid, Some(&oid)).await?;
		Ok(aid)
	}

	async fn like(&self, uid: String, activity: serde_json::Value) -> crate::Result<String> {
		let aid = self.aid(&uuid::Uuid::new_v4().to_string());
		let activity_targets = activity.addressed();
		let oid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let obj_model = self.fetch_object(&oid).await?;

		let internal_uid = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;

		if model::like::Entity::find_by_uid_oid(internal_uid, obj_model.internal)
			.any(self.db())
			.await?
		{
			return Err(UpubError::not_modified());
		}

		let activity_model = self.insert_activity(
			activity
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now())),
			Some(self.domain().to_string()),
		).await?;

		self.process_like(internal_uid, obj_model.internal, activity_model.internal, chrono::Utc::now()).await?;

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
				.set_published(Some(chrono::Utc::now()))
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
				.set_published(Some(chrono::Utc::now()))
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
				.set_published(Some(chrono::Utc::now()))
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
		let internal_uid = model::actor::Entity::ap_to_internal(&uid, self.db()).await?;
		let old_aid = activity.object().id().ok_or_else(UpubError::bad_request)?;
		let old_activity = model::activity::Entity::find_by_ap_id(&old_aid)
			.one(self.db())
			.await?
			.ok_or_else(UpubError::not_found)?;
		if old_activity.actor != uid {
			return Err(UpubError::forbidden());
		}

		let activity_model = self.insert_activity(
			activity.clone()
				.set_id(Some(&aid))
				.set_actor(Node::link(uid.clone()))
				.set_published(Some(chrono::Utc::now())),
			Some(self.domain().to_string())
		).await?;

		let targets = self.expand_addressing(activity.addressed()).await?;
		self.process_undo(internal_uid, activity).await?;

		self.address_to(Some(activity_model.internal), None, &targets).await?;
		self.deliver_to(&activity_model.id, &uid, &targets).await?;

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
				.set_published(Some(chrono::Utc::now()))
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
				.set_published(Some(chrono::Utc::now()))
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
				.set_published(Some(chrono::Utc::now()))
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
