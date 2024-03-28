use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{EntityTrait, IntoActiveModel, Order, QueryOrder, QuerySelect, Set};

use crate::{activitypub::{jsonld::LD, CreationResult, JsonLD, Pagination}, activitystream::{object::{activity::{accept::AcceptType, Activity, ActivityMut, ActivityType}, Addressed, ObjectMut}, Base, BaseMut, BaseType, Node, ObjectType}, auth::{AuthIdentity, Identity}, errors::UpubError, model, server::Context, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	Ok(JsonLD(
		ctx.ap_collection(&url!(ctx, "/users/{id}/outbox"), None).ld_context()
	))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(_auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	// let mut conditions = Condition::any()
	// 	.add(model::addressing::Column::Actor.eq(PUBLIC_TARGET));

	// if let Identity::User(ref x) = auth {
	// 	conditions = conditions.add(model::addressing::Column::Actor.eq(x));
	// }

	// if let Identity::Server(ref x) = auth {
	// 	conditions = conditions.add(model::addressing::Column::Server.eq(x));
	// }

	match model::activity::Entity::find()
		.find_also_related(model::object::Entity)
		.order_by(model::activity::Column::Published, Order::Desc)
		.limit(limit)
		.offset(offset)
		.all(ctx.db()).await
	{
		Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
		Ok(items) => {
			Ok(JsonLD(
				ctx.ap_collection_page(
					&url!(ctx, "/users/{id}/outbox/page"),
					offset, limit,
					items
						.into_iter()
						.map(|(a, o)| {
							let oid = a.object.clone();
							Node::object(
								super::super::activity::ap_activity(a)
									.set_object(match o {
										Some(o) => Node::object(super::super::object::ap_object(o)),
										None    => Node::maybe_link(oid),
									})
							)
						})
						.collect()
				).ld_context()
			))
		},
	}
}

pub async fn post(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>,
) -> Result<CreationResult, UpubError> {
	match auth {
		Identity::Anonymous => Err(StatusCode::UNAUTHORIZED.into()),
		Identity::Remote(_) => Err(StatusCode::NOT_IMPLEMENTED.into()),
		Identity::Local(uid) => if ctx.uid(id.clone()) == uid {
			match activity.base_type() {
				None => Err(StatusCode::BAD_REQUEST.into()),
				Some(BaseType::Link(_)) => Err(StatusCode::UNPROCESSABLE_ENTITY.into()),

				// Some(BaseType::Object(ObjectType::Note)) => {
				// },

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
					let Some(object) = activity.object().get().map(|x| x.underlying_json_object()) else {
						return Err(StatusCode::BAD_REQUEST.into());
					};
					let oid = ctx.oid(uuid::Uuid::new_v4().to_string());
					let aid = ctx.aid(uuid::Uuid::new_v4().to_string());
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
						.exec(ctx.db()).await?;
					model::activity::Entity::insert(activity_model.into_active_model())
						.exec(ctx.db()).await?;

					let addressed = ctx.expand_addressing(&uid, activity_targets).await?;
					ctx.address_to(&aid, Some(&oid), &addressed).await?;
					ctx.deliver_to(&aid, &uid, &addressed).await?;
					Ok(CreationResult(aid))
				},

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => {
					let aid = ctx.aid(uuid::Uuid::new_v4().to_string());
					let activity_targets = activity.addressed();
					let Some(oid) = activity.object().id().map(|x| x.to_string()) else {
						return Err(StatusCode::BAD_REQUEST.into());
					};
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
					model::like::Entity::insert(like_model).exec(ctx.db()).await?;
					model::activity::Entity::insert(activity_model.into_active_model())
						.exec(ctx.db()).await?;

					let addressed = ctx.expand_addressing(&uid, activity_targets).await?;
					ctx.address_to(&aid, None, &addressed).await?;
					ctx.deliver_to(&aid, &uid, &addressed).await?;
					Ok(CreationResult(aid))
				},

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => {
					let aid = ctx.aid(uuid::Uuid::new_v4().to_string());
					let activity_targets = activity.addressed();
					if activity.object().id().is_none() {
						return Err(StatusCode::BAD_REQUEST.into());
					}

					let activity_model = model::activity::Model::new(
						&activity
							.set_id(Some(&aid))
							.set_actor(Node::link(uid.clone()))
							.set_published(Some(chrono::Utc::now()))
					)?;
					model::activity::Entity::insert(activity_model.into_active_model())
						.exec(ctx.db()).await?;

					let addressed = ctx.expand_addressing(&uid, activity_targets).await?;
					ctx.address_to(&aid, None, &addressed).await?;
					ctx.deliver_to(&aid, &uid, &addressed).await?;
					Ok(CreationResult(aid))
				},

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Undo))) => {
					let aid = ctx.aid(uuid::Uuid::new_v4().to_string());
					let activity_targets = activity.addressed();
					{
						let Some(old_aid) = activity.object().id() else {
							return Err(StatusCode::BAD_REQUEST.into());
						};
						let Some(old_activity) = model::activity::Entity::find_by_id(old_aid)
							.one(ctx.db()).await?
						else {
							return Err(StatusCode::NOT_FOUND.into());
						};
						if old_activity.actor != uid {
							return Err(StatusCode::FORBIDDEN.into());
						}
						match old_activity.activity_type {
							ActivityType::Like => {
								model::like::Entity::delete(model::like::ActiveModel {
									actor: Set(old_activity.actor), likes: Set(old_activity.object.unwrap_or("".into())),
									..Default::default()
								}).exec(ctx.db()).await?;
							},
							ActivityType::Follow => {
								model::relation::Entity::delete(model::relation::ActiveModel {
									follower: Set(old_activity.actor), following: Set(old_activity.object.unwrap_or("".into())),
									..Default::default()
								}).exec(ctx.db()).await?;
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
					model::activity::Entity::insert(activity_model.into_active_model()).exec(ctx.db()).await?;

					let addressed = ctx.expand_addressing(&uid, activity_targets).await?;
					ctx.address_to(&aid, None, &addressed).await?;
					ctx.deliver_to(&aid, &uid, &addressed).await?;
					Ok(CreationResult(aid))
				},

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Accept(AcceptType::Accept)))) => {
					let aid = ctx.aid(uuid::Uuid::new_v4().to_string());
					let activity_targets = activity.addressed();
					if activity.object().id().is_none() {
						return Err(StatusCode::BAD_REQUEST.into());
					}
					let Some(accepted_id) = activity.object().id() else {
						return Err(StatusCode::BAD_REQUEST.into());
					};
					let Some(accepted_activity) = model::activity::Entity::find_by_id(accepted_id)
						.one(ctx.db()).await?
					else {
						return Err(StatusCode::NOT_FOUND.into());
					};

					match accepted_activity.activity_type {
						ActivityType::Follow => {
							model::relation::Entity::insert(
								model::relation::ActiveModel {
									follower: Set(accepted_activity.actor), following: Set(uid.clone()),
									..Default::default()
								}
							).exec(ctx.db()).await?;
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
						.exec(ctx.db()).await?;

					let addressed = ctx.expand_addressing(&uid, activity_targets).await?;
					ctx.address_to(&aid, None, &addressed).await?;
					ctx.deliver_to(&aid, &uid, &addressed).await?;
					Ok(CreationResult(aid))
				},

				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Reject(RejectType::Reject)))) => {
				// },

				Some(_) => Err(StatusCode::NOT_IMPLEMENTED.into()),
			}
		} else {
			Err(StatusCode::FORBIDDEN.into())
		}
	}
}
