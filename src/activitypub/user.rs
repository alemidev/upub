use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{activitystream::{object::{activity::{Activity, ActivityType}, ObjectType}, Base, BaseType, Node}, model::{activity, object, user}};

pub async fn list(State(_db) : State<Arc<DatabaseConnection>>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}

pub async fn view(State(db) : State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	match user::Entity::find_by_id(super::uri_id(id)).one(&*db).await {
		Ok(Some(user)) => Ok(Json(user.underlying_json_object())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for user: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

pub async fn outbox(State(db): State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	let uri = super::uri_id(id);
	match activity::Entity::find()
		.filter(Condition::all().add(activity::Column::Actor.eq(uri)))
		.all(&*db).await
	{
		Ok(_x) => todo!(),
		Err(_e) => todo!(),
	}
}

pub async fn inbox(
	State(db): State<Arc<DatabaseConnection>>,
	Path(_id): Path<String>,
	Json(object): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
	match object.base_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(BaseType::Link(_x)) => Err(StatusCode::UNPROCESSABLE_ENTITY), // we could but not yet
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => Err(StatusCode::UNPROCESSABLE_ENTITY),
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
			let Ok(activity_entity) = activity::Model::new(&object) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Node::Object(obj) = object.object() else {
				// TODO we could process non-embedded activities or arrays but im lazy rn
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Ok(obj_entity) = object::Model::new(&*obj) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			object::Entity::insert(obj_entity.into_active_model())
				.exec(&*db)
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			activity::Entity::insert(activity_entity.into_active_model())
				.exec(&*db)
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(Json(serde_json::Value::Null)) // TODO hmmmmmmmmmmm not the best value to return....
		},
		Some(BaseType::Object(ObjectType::Activity(_x))) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(_x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}
