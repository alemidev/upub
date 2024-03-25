use axum::{extract::{Path, State}, http::StatusCode, Json};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{activitypub::JsonLD, activitystream::{object::{activity::{Activity, ActivityType}, Addressed, ObjectType}, Base, BaseType, Node}, errors::LoggableError, model::{self, activity, addressing, object}, server::Context};

pub async fn post(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	Json(object): Json<serde_json::Value>
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match object.base_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(BaseType::Link(_x)) => {
			tracing::warn!("skipping remote activity: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY) // we could but not yet
		},
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => {
			tracing::warn!("skipping unprocessable base activity: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY) // won't ingest useless stuff
		},
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Delete))) => {
			// TODO verify the signature before just deleting lmao
			let oid = object.object().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			// TODO maybe we should keep the tombstone?
			model::user::Entity::delete_by_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from users");
			model::activity::Entity::delete_by_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from activities");
			model::object::Entity::delete_by_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from objects");
			Ok(JsonLD(serde_json::Value::Null))
		},
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => {
			let Ok(activity_entity) = activity::Model::new(&object) else {
				tracing::warn!("could not serialize activity: {}", serde_json::to_string_pretty(&object).unwrap());
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			tracing::info!("{} wants to follow {}", activity_entity.actor, activity_entity.object.as_deref().unwrap_or("<no-one???>"));
			activity::Entity::insert(activity_entity.into_active_model())
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(JsonLD(serde_json::Value::Null))
		},
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => {
			let aid = object.actor().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			let oid = object.object().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			let like = model::like::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				actor: sea_orm::Set(aid.clone()),
				likes: sea_orm::Set(oid.clone()),
				date: sea_orm::Set(chrono::Utc::now()),
			};
			match model::like::Entity::insert(like).exec(ctx.db()).await {
				Err(sea_orm::DbErr::RecordNotInserted) => Err(StatusCode::NOT_MODIFIED),
				Err(sea_orm::DbErr::Exec(_)) => Err(StatusCode::NOT_MODIFIED), // bad fix for sqlite
				Err(e) => {
					tracing::error!("unexpected error procesing like from {aid} to {oid}: {e}");
					Err(StatusCode::INTERNAL_SERVER_ERROR)
				}
				Ok(_) => {
					match model::object::Entity::update_many()
						.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
						.filter(model::object::Column::Id.eq(oid.clone()))
						.exec(ctx.db())
						.await
					{
						Err(e) => {
							tracing::error!("unexpected error incrementing object {oid} like counter: {e}");
							Err(StatusCode::INTERNAL_SERVER_ERROR)
						},
						Ok(_) => {
							tracing::info!("{} liked {}", aid, oid);
							Ok(JsonLD(serde_json::Value::Null))
						}
					}
				},
			}
		},
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
			let Ok(activity_entity) = activity::Model::new(&object) else {
				tracing::warn!("could not serialize activity: {}", serde_json::to_string_pretty(&object).unwrap());
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Node::Object(obj) = object.object() else {
				// TODO we could process non-embedded activities or arrays but im lazy rn
				tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&object).unwrap());
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Ok(obj_entity) = object::Model::new(&*obj) else {
				tracing::warn!("coult not serialize object: {}", serde_json::to_string_pretty(&object).unwrap());
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			tracing::info!("processing Create activity by {} for {}", activity_entity.actor, activity_entity.object.as_deref().unwrap_or("<embedded>"));
			let object_id = obj_entity.id.clone();
			let activity_id = activity_entity.id.clone();
			object::Entity::insert(obj_entity.into_active_model())
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			activity::Entity::insert(activity_entity.into_active_model())
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			addressing::Entity::insert_many(
				object.addressed()
					.into_iter()
					.map(|actor|
						addressing::ActiveModel{
							id: sea_orm::ActiveValue::NotSet,
							actor: sea_orm::Set(actor),
							activity: sea_orm::Set(activity_id.clone()),
							object: sea_orm::Set(Some(object_id.clone())),
							published: sea_orm::Set(chrono::Utc::now()),
						}
					)
			)
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(JsonLD(serde_json::Value::Null)) // TODO hmmmmmmmmmmm not the best value to return....
		},
		Some(BaseType::Object(ObjectType::Activity(_x))) => {
			tracing::info!("received unimplemented activity on inbox: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::NOT_IMPLEMENTED)
		},
		Some(_x) => {
			tracing::warn!("ignoring non-activity object in inbox: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY)
		}
	}
}

pub async fn get() -> StatusCode {
	StatusCode::NOT_IMPLEMENTED
}
