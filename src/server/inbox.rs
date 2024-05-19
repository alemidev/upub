use apb::{target::Addressed, Activity, Base, Object};
use reqwest::StatusCode;
use sea_orm::{sea_query::Expr, ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns, Set};

use crate::{errors::{LoggableError, UpubError}, model::{self, FieldError}};

use super::{fetcher::Fetcher, Context};


#[axum::async_trait]
impl apb::server::Inbox for Context {
	type Error = UpubError;
	type Activity = serde_json::Value;

	async fn create(&self, server: String, activity: serde_json::Value) -> crate::Result<()> {
		let activity_model = model::activity::Model::new(&activity)?;
		let aid = activity_model.id.clone();
		let Some(object_node) = activity.object().extract() else {
			// TODO we could process non-embedded activities or arrays but im lazy rn
			tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&activity).unwrap());
			return Err(UpubError::unprocessable());
		};
		let mut object_model = model::object::Model::new(&object_node)?;
		let oid = object_model.id.clone();
		let uid = object_model.attributed_to.clone();
		// make sure we're allowed to edit this object
		if let Some(object_author) = &object_model.attributed_to {
			if server != Context::server(object_author) {
				return Err(UpubError::forbidden());
			}
		} else if server != Context::server(&object_model.id) {
			return Err(UpubError::forbidden());
		};
		// fix context also for remote posts
		// TODO this is not really appropriate because we're mirroring incorrectly remote objects, but
		// it makes it SOO MUCH EASIER for us to fetch threads and stuff, so we're filling it for them
		match (&object_model.in_reply_to, &object_model.context) {
			(Some(reply_id), None) => // get context from replied object
				object_model.context = self.fetch_object(reply_id).await?.context,
			(None, None) => // generate a new context
				object_model.context = Some(crate::url!(self, "/context/{}", uuid::Uuid::new_v4().to_string())),
			(_, Some(_)) => {}, // leave it as set by user
		}
		// update replies counter
		if let Some(ref in_reply_to) = object_model.in_reply_to {
			if self.fetch_object(in_reply_to).await.is_ok() {
				model::object::Entity::update_many()
					.filter(model::object::Column::Id.eq(in_reply_to))
					.col_expr(model::object::Column::Comments, Expr::col(model::object::Column::Comments).add(1))
					.exec(self.db())
					.await?;
			}
		}
		model::object::Entity::insert(object_model.into_active_model()).exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.into_active_model()).exec(self.db()).await?;
		for attachment in object_node.attachment() {
			let attachment_model = model::attachment::ActiveModel::new(&attachment, oid.clone(), None)?;
			model::attachment::Entity::insert(attachment_model)
				.exec(self.db())
				.await?;
		}
		// lemmy sends us an image field in posts, treat it like an attachment i'd say
		if let Some(img) = object_node.image().get() {
			// TODO lemmy doesnt tell us the media type but we use it to display the thing...
			let img_url = img.url().id().unwrap_or_default();
			let media_type = if img_url.ends_with("png") {
				Some("image/png".to_string())
			} else if img_url.ends_with("webp") {
				Some("image/webp".to_string())
			} else if img_url.ends_with("jpeg") || img_url.ends_with("jpg") {
				Some("image/jpeg".to_string())
			} else {
				None
			};
			let attachment_model = model::attachment::ActiveModel::new(img, oid.clone(), media_type)?;
			model::attachment::Entity::insert(attachment_model)
				.exec(self.db())
				.await?;
		}
		// TODO can we even receive anonymous objects?
		if let Some(object_author) = uid {
			model::user::Entity::update_many()
				.col_expr(model::user::Column::StatusesCount, Expr::col(model::user::Column::StatusesCount).add(1))
				.filter(model::user::Column::Id.eq(&object_author))
				.exec(self.db())
				.await?;
		}
		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(Some(&aid), Some(&oid), &expanded_addressing).await?;
		tracing::info!("{} posted {}", aid, oid);
		Ok(())
	}

	async fn like(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		let aid = activity.id().ok_or(UpubError::bad_request())?;
		let uid = activity.actor().id().ok_or(UpubError::bad_request())?;
		let object_uri = activity.object().id().ok_or(UpubError::bad_request())?;
		let obj = self.fetch_object(&object_uri).await?;
		let oid = obj.id;
		let like = model::like::ActiveModel {
			id: sea_orm::ActiveValue::NotSet,
			actor: sea_orm::Set(uid.clone()),
			likes: sea_orm::Set(oid.clone()),
			date: sea_orm::Set(activity.published().unwrap_or(chrono::Utc::now())),
		};
		match model::like::Entity::insert(like).exec(self.db()).await {
			Err(sea_orm::DbErr::RecordNotInserted) => Err(UpubError::not_modified()),
			Err(sea_orm::DbErr::Exec(_)) => Err(UpubError::not_modified()), // bad fix for sqlite
			Err(e) => {
				tracing::error!("unexpected error procesing like from {uid} to {oid}: {e}");
				Err(UpubError::internal_server_error())
			}
			Ok(_) => {
				let activity_model = model::activity::Model::new(&activity)?.into_active_model();
				model::activity::Entity::insert(activity_model)
					.exec(self.db())
					.await?;
				let mut expanded_addressing = self.expand_addressing(activity.addressed()).await?;
				if expanded_addressing.is_empty() { // WHY MASTODON!!!!!!!
					expanded_addressing.push(
						model::object::Entity::find_by_id(&oid)
							.select_only()
							.select_column(model::object::Column::AttributedTo)
							.into_tuple::<String>()
							.one(self.db())
							.await?
							.ok_or_else(UpubError::not_found)?
						);
				}
				self.address_to(Some(aid), None, &expanded_addressing).await?;
				model::object::Entity::update_many()
					.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
					.filter(model::object::Column::Id.eq(oid.clone()))
					.exec(self.db())
					.await?;
				tracing::info!("{} liked {}", uid, oid);
				Ok(())
			},
		}
	}

	async fn follow(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		let activity_model = model::activity::Model::new(&activity)?;
		let aid = activity_model.id.clone();
		let target_user_uri = activity_model.object
			.as_deref()
			.ok_or_else(UpubError::bad_request)?
			.to_string();
		let usr = self.fetch_user(&target_user_uri).await?;
		let target_user_id = usr.id;
		tracing::info!("{} wants to follow {}", activity_model.actor, target_user_id);
		model::activity::Entity::insert(activity_model.into_active_model())
			.exec(self.db()).await?;
		let mut expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		if !expanded_addressing.contains(&target_user_id) {
			expanded_addressing.push(target_user_id);
		}
		self.address_to(Some(&aid), None, &expanded_addressing).await?;
		Ok(())
	}

	async fn accept(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		// TODO what about TentativeAccept
		let activity_model = model::activity::Model::new(&activity)?;

		if let Some(mut r) = model::relay::Entity::find_by_id(&activity_model.actor)
			.one(self.db())
			.await?
		{
			r.accepted = true;
			model::relay::Entity::update(r.into_active_model()).exec(self.db()).await?;
			model::activity::Entity::insert(activity_model.clone().into_active_model())
				.exec(self.db())
				.await?;
			tracing::info!("relay {} is now broadcasting to us", activity_model.actor);
			return Ok(());
		}

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
		model::user::Entity::update_many()
			.col_expr(
				model::user::Column::FollowingCount,
				Expr::col(model::user::Column::FollowingCount).add(1)
			)
			.filter(model::user::Column::Id.eq(&follow_activity.actor))
			.exec(self.db())
			.await?;
		model::relation::Entity::insert(
			model::relation::ActiveModel {
				follower: Set(follow_activity.actor.clone()),
				following: Set(activity_model.actor),
				..Default::default()
			}
		).exec(self.db()).await?;

		let mut expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		if !expanded_addressing.contains(&follow_activity.actor) {
			expanded_addressing.push(follow_activity.actor);
		}
		self.address_to(Some(&activity_model.id), None, &expanded_addressing).await?;
		Ok(())
	}

	async fn reject(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
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

		let mut expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		if !expanded_addressing.contains(&follow_activity.actor) {
			expanded_addressing.push(follow_activity.actor);
		}
		self.address_to(Some(&activity_model.id), None, &expanded_addressing).await?;
		Ok(())
	}

	async fn delete(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		// TODO verify the signature before just deleting lmao
		let oid = activity.object().id().ok_or(UpubError::bad_request())?;
		tracing::debug!("deleting '{oid}'"); // this is so spammy wtf!
		// TODO maybe we should keep the tombstone?
		model::user::Entity::delete_by_id(&oid).exec(self.db()).await.info_failed("failed deleting from users");
		model::activity::Entity::delete_by_id(&oid).exec(self.db()).await.info_failed("failed deleting from activities");
		model::object::Entity::delete_by_id(&oid).exec(self.db()).await.info_failed("failed deleting from objects");
		Ok(())
	}

	async fn update(&self, server: String, activity: serde_json::Value) -> crate::Result<()> {
		let activity_model = model::activity::Model::new(&activity)?;
		let aid = activity_model.id.clone();
		let Some(object_node) = activity.object().extract() else {
			// TODO we could process non-embedded activities or arrays but im lazy rn
			tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&activity).unwrap());
			return Err(UpubError::unprocessable());
		};
		let Some(oid) = object_node.id().map(|x| x.to_string()) else {
			return Err(UpubError::bad_request());
		};
		// make sure we're allowed to edit this object
		if let Some(object_author) = object_node.attributed_to().id() {
			if server != Context::server(&object_author) {
				return Err(UpubError::forbidden());
			}
		} else if server != Context::server(&oid) {
			return Err(UpubError::forbidden());
		};
		match object_node.object_type() {
			Some(apb::ObjectType::Actor(_)) => {
				// TODO oof here is an example of the weakness of this model, we have to go all the way
				// back up to serde_json::Value because impl Object != impl Actor
				let actor_model = model::user::Model::new(&object_node)?;
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
				let object_model = model::object::Model::new(&object_node)?;
				let mut  update_model = object_model.into_active_model();
				update_model.updated = sea_orm::Set(Some(chrono::Utc::now()));
				update_model.reset(model::object::Column::Name);
				update_model.reset(model::object::Column::Summary);
				update_model.reset(model::object::Column::Content);
				update_model.reset(model::object::Column::Sensitive);
				model::object::Entity::update(update_model)
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
		self.address_to(Some(&aid), Some(&oid), &expanded_addressing).await?;
		Ok(())
	}

	async fn undo(&self, server: String, activity: serde_json::Value) -> crate::Result<()> {
		let uid = activity.actor().id().ok_or_else(UpubError::bad_request)?;
		// TODO in theory we could work with just object_id but right now only accept embedded
		let undone_activity = activity.object().extract().ok_or_else(UpubError::bad_request)?;
		let undone_aid = undone_activity.id().ok_or_else(UpubError::bad_request)?;
		let undone_object_uri = undone_activity.object().id().ok_or_else(UpubError::bad_request)?;
		let activity_type = undone_activity.activity_type().ok_or_else(UpubError::bad_request)?;
		let undone_activity_author = undone_activity.actor().id().ok_or_else(UpubError::bad_request)?;

		// can't undo activities from remote actors!
		if server != Context::server(&undone_activity_author) {
			return Err(UpubError::forbidden());
		};

		let obj = self.fetch_object(&undone_object_uri).await?;
		let undone_object_id = obj.id;

		match activity_type {
			apb::ActivityType::Like => {
				model::like::Entity::delete_many()
					.filter(
						Condition::all()
							.add(model::like::Column::Actor.eq(&uid))
							.add(model::like::Column::Likes.eq(&undone_object_id))
					)
					.exec(self.db())
					.await?;
				model::object::Entity::update_many()
					.filter(model::object::Column::Id.eq(&undone_object_id))
					.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).sub(1))
					.exec(self.db())
					.await?;
			},
			apb::ActivityType::Follow => {
				model::relation::Entity::delete_many()
					.filter(
						Condition::all()
							.add(model::relation::Column::Follower.eq(&uid))
							.add(model::relation::Column::Following.eq(&undone_object_id))
					)
					.exec(self.db())
					.await?;
			},
			_ => {
				tracing::error!("received 'Undo' for unimplemented activity: {}", serde_json::to_string_pretty(&activity).unwrap());
				return Err(StatusCode::NOT_IMPLEMENTED.into());
			},
		}

		model::activity::Entity::delete_by_id(undone_aid).exec(self.db()).await?;

		Ok(())

	}
	
	async fn announce(&self, _: String, activity: serde_json::Value) -> crate::Result<()> {
		let activity_model = model::activity::Model::new(&activity)?;
		let Some(object_uri) = &activity_model.object else {
			return Err(FieldError("object").into());
		};
		let obj = self.fetch_object(object_uri).await?;
		let oid = obj.id;

		// relays send us activities as Announce, but we don't really want to count those towards the
		// total shares count of an object, so just fetch the object and be done with it
		if self.is_relay(&activity_model.actor) {
			tracing::info!("relay {} broadcasted {}", activity_model.actor, oid);
			return Ok(())
		}

		let share = model::share::ActiveModel {
			id: sea_orm::ActiveValue::NotSet,
			actor: sea_orm::Set(activity_model.actor.clone()),
			shares: sea_orm::Set(oid.clone()),
			date: sea_orm::Set(activity.published().unwrap_or(chrono::Utc::now())),
		};

		let expanded_addressing = self.expand_addressing(activity.addressed()).await?;
		self.address_to(Some(&activity_model.id), None, &expanded_addressing).await?;
		model::share::Entity::insert(share)
			.exec(self.db()).await?;
		model::activity::Entity::insert(activity_model.clone().into_active_model())
			.exec(self.db())
			.await?;
		model::object::Entity::update_many()
			.col_expr(model::object::Column::Shares, Expr::col(model::object::Column::Shares).add(1))
			.filter(model::object::Column::Id.eq(oid.clone()))
			.exec(self.db())
			.await?;

		tracing::info!("{} shared {}", activity_model.actor, oid);
		Ok(())
	}
}
