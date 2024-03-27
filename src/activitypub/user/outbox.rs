use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json};
use sea_orm::{ColumnTrait, Condition, DbErr, EntityTrait, IntoActiveModel, Order, QueryFilter, QueryOrder, QuerySelect, SelectColumns, Set};

use crate::{activitypub::{jsonld::LD, JsonLD, Pagination, PUBLIC_TARGET}, activitystream::{object::{activity::{accept::AcceptType, Activity, ActivityMut, ActivityType}, collection::{page::CollectionPageMut, CollectionMut, CollectionType}, Addressed, ObjectMut}, Base, BaseMut, BaseType, Node, ObjectType}, auth::{AuthIdentity, Identity}, model::{self, activity, object, FieldError}, server::Context, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	Ok(JsonLD(
		serde_json::Value::new_object()
			.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
			.set_collection_type(Some(CollectionType::OrderedCollection))
			.set_first(Node::link(url!(ctx, "/users/{id}/outbox/page")))
			.ld_context()
	))
}

#[derive(Debug, thiserror::Error)]
pub enum UpubError {
	#[error("database error: {0}")]
	Database(#[from] sea_orm::DbErr),

	#[error("api returned {0}")]
	Status(StatusCode),

	#[error("missing field: {0}")]
	Field(#[from] FieldError),

	#[error("openssl error: {0}")]
	OpenSSL(#[from] openssl::error::ErrorStack),

	#[error("fetch error: {0}")]
	Reqwest(#[from] reqwest::Error),
}

impl From<StatusCode> for UpubError {
	fn from(value: StatusCode) -> Self {
		UpubError::Status(value)
	}
}

impl IntoResponse for UpubError {
	fn into_response(self) -> axum::response::Response {
		(StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
	}
}

pub struct CreationResult(pub String);
impl IntoResponse for CreationResult {
	fn into_response(self) -> axum::response::Response {
		(
			StatusCode::CREATED,
			[("Location", self.0.as_str())]
		)
			.into_response()
	}
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

	match activity::Entity::find()
		.find_also_related(object::Entity)
		.order_by(activity::Column::Published, Order::Desc)
		.limit(limit)
		.offset(offset)
		.all(ctx.db()).await
	{
		Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
		Ok(items) => {
			Ok(JsonLD(
				serde_json::Value::new_object()
					// TODO set id, calculate uri from given args
					.set_id(Some(&url!(ctx, "/users/{id}/outbox/page?offset={offset}")))
					.set_collection_type(Some(CollectionType::OrderedCollectionPage))
					.set_part_of(Node::link(url!(ctx, "/users/{id}/outbox")))
					.set_next(Node::link(url!(ctx, "/users/{id}/outbox/page?offset={}", limit+offset)))
					.set_ordered_items(Node::array(
						items
							.into_iter()
							.map(|(a, o)| 
								super::super::activity::ap_activity(a)
									.set_object(Node::maybe_object(o.map(super::super::object::ap_object)))
							)
							.collect()
					))
					.ld_context()
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

impl Context {
	async fn expand_addressing(&self, uid: &str, mut targets: Vec<String>) -> Result<Vec<String>, DbErr> {
		let following_addr = format!("{uid}/followers");
		if let Some(i) = targets.iter().position(|x| x == &following_addr) {
			targets.remove(i);
			model::relation::Entity::find()
				.filter(Condition::all().add(model::relation::Column::Following.eq(uid.to_string())))
				.select_column(model::relation::Column::Follower)
				.into_tuple::<String>()
				.all(self.db())
				.await?
				.into_iter()
				.for_each(|x| targets.push(x));
		}
		Ok(targets)
	}

	async fn address_to(&self, aid: &str, oid: Option<&str>, targets: &[String]) -> Result<(), DbErr> {
		let addressings : Vec<model::addressing::ActiveModel> = targets
			.iter()
			.map(|to| model::addressing::ActiveModel {
				server: Set(Context::server(to)),
				actor: Set(to.to_string()),
				activity: Set(aid.to_string()),
				object: Set(oid.map(|x| x.to_string())),
				published: Set(chrono::Utc::now()),
				..Default::default()
			})
			.collect();

		model::addressing::Entity::insert_many(addressings)
			.exec(self.db())
			.await?;

		Ok(())
	}

	async fn deliver_to(&self, aid: &str, from: &str, targets: &[String]) -> Result<(), DbErr> {
		let deliveries : Vec<model::delivery::ActiveModel> = targets
			.iter()
			.filter(|to| Context::server(to) != self.base())
			.filter(|to| to != &PUBLIC_TARGET)
			.map(|to| model::delivery::ActiveModel {
				actor: Set(from.to_string()),
				// TODO we should resolve each user by id and check its inbox because we can't assume
				// it's /users/{id}/inbox for every software, but oh well it's waaaaay easier now
				target: Set(format!("{}/inbox", to)),
				activity: Set(aid.to_string()),
				created: Set(chrono::Utc::now()),
				not_before: Set(chrono::Utc::now()),
				attempt: Set(0),
				..Default::default()
			})
			.collect();

		model::delivery::Entity::insert_many(deliveries)
			.exec(self.db())
			.await?;

		Ok(())
	}
}

