use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json};
use sea_orm::{ColumnTrait, Condition, EntityTrait, IntoActiveModel, Order, QueryFilter, QueryOrder, QuerySelect, SelectColumns, Set};

use crate::{activitypub::{jsonld::LD, JsonLD, Pagination, PUBLIC_TARGET}, activitystream::{object::{activity::{Activity, ActivityMut, ActivityType}, collection::{page::CollectionPageMut, CollectionMut, CollectionType}, Addressed}, Base, BaseMut, BaseType, Node, ObjectType}, auth::{AuthIdentity, Identity}, model::{self, activity, object, FieldError}, server::Context, url};

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
					let oid = uuid::Uuid::new_v4().to_string();
					let aid = uuid::Uuid::new_v4().to_string();
					let mut object_model = model::object::Model::new(&object)?;
					let mut activity_model = model::activity::Model::new(&activity)?;
					object_model.id = oid.clone();
					object_model.to = activity_model.to.clone();
					object_model.bto = activity_model.bto.clone();
					object_model.cc = activity_model.cc.clone();
					object_model.bcc = activity_model.bcc.clone();
					object_model.attributed_to = Some(uid.clone());
					object_model.published = chrono::Utc::now();
					activity_model.id = aid.clone();
					activity_model.published = chrono::Utc::now();
					activity_model.actor = uid.clone();
					activity_model.object = Some(oid.clone());

					model::object::Entity::insert(object_model.into_active_model())
						.exec(ctx.db()).await?;

					model::activity::Entity::insert(activity_model.into_active_model())
						.exec(ctx.db()).await?;

					let mut addressed = activity.addressed();
					let followers = url!(ctx, "/users/{id}/followers"); // TODO maybe can be done better?
					if let Some(i) = addressed.iter().position(|x| x == &followers) {
						addressed.remove(i);
						model::relation::Entity::find()
							.filter(Condition::all().add(model::relation::Column::Following.eq(uid.clone())))
							.select_column(model::relation::Column::Follower)
							.into_tuple::<String>()
							.all(ctx.db())
							.await?
							.into_iter()
							.for_each(|x| addressed.push(x));
					}

					let addressings : Vec<model::addressing::ActiveModel> = addressed
						.iter()
						.map(|to| model::addressing::ActiveModel {
							server: Set(Context::server(&uid)),
							actor: Set(to.to_string()),
							activity: Set(aid.clone()),
							object: Set(Some(oid.clone())),
							..Default::default()
						})
						.collect();

					model::addressing::Entity::insert_many(addressings)
						.exec(ctx.db()).await?;

					let deliveries : Vec<model::delivery::ActiveModel> = addressed
						.iter()
						.filter(|to| Context::server(to) != ctx.base())
						.filter(|to| to != &PUBLIC_TARGET)
						.map(|to| model::delivery::ActiveModel {
							// TODO we should resolve each user by id and check its inbox because we can't assume
							// it's /users/{id}/inbox for every software, but oh well it's waaaaay easier now
							actor: Set(uid.clone()),
							target: Set(format!("{}/inbox", to)),
							activity: Set(aid.clone()),
							created: Set(chrono::Utc::now()),
							not_before: Set(chrono::Utc::now()),
							attempt: Set(0),
							..Default::default()
						})
						.collect();

					model::delivery::Entity::insert_many(deliveries)
						.exec(ctx.db())
						.await?;

					Ok(CreationResult(aid))
				},
				Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => {
					let aid = uuid::Uuid::new_v4().to_string();
					let mut activity_model = model::activity::Model::new(&activity)?;
					activity_model.id = aid.clone();
					activity_model.published = chrono::Utc::now();
					activity_model.actor = uid.clone();

					model::activity::Entity::insert(activity_model.into_active_model())
						.exec(ctx.db()).await?;

					let mut addressed = activity.addressed();
					let followers = url!(ctx, "/users/{id}/followers"); // TODO maybe can be done better?
					if let Some(i) = addressed.iter().position(|x| x == &followers) {
						addressed.remove(i);
						model::relation::Entity::find()
							.filter(Condition::all().add(model::relation::Column::Following.eq(uid.clone())))
							.select_column(model::relation::Column::Follower)
							.into_tuple::<String>()
							.all(ctx.db())
							.await?
							.into_iter()
							.for_each(|x| addressed.push(x));
					}

					let addressings : Vec<model::addressing::ActiveModel> = addressed
						.iter()
						.map(|to| model::addressing::ActiveModel {
							server: Set(Context::server(&uid)),
							actor: Set(to.to_string()),
							activity: Set(aid.clone()),
							object: Set(None),
							..Default::default()
						})
						.collect();

					model::addressing::Entity::insert_many(addressings)
						.exec(ctx.db())
						.await?;

					let deliveries : Vec<model::delivery::ActiveModel> = addressed
						.iter()
						.filter(|to| Context::server(to) != ctx.base())
						.filter(|to| to != &PUBLIC_TARGET)
						.map(|to| model::delivery::ActiveModel {
							// TODO we should resolve each user by id and check its inbox because we can't assume
							// it's /users/{id}/inbox for every software, but oh well it's waaaaay easier now
							actor: Set(uid.clone()),
							target: Set(format!("{}/inbox", to)),
							activity: Set(aid.clone()),
							created: Set(chrono::Utc::now()),
							not_before: Set(chrono::Utc::now()),
							attempt: Set(0),
							..Default::default()
						})
						.collect();

					model::delivery::Entity::insert_many(deliveries)
						.exec(ctx.db())
						.await?;

					Ok(CreationResult(aid))
				},
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Undo))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Accept(AcceptType::Accept)))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Reject(RejectType::Reject)))) => {
				// },
				Some(_) => Err(StatusCode::NOT_IMPLEMENTED.into()),
			}
		} else {
			Err(StatusCode::FORBIDDEN.into())
		}
	}
}
