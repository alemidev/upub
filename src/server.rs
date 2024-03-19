use std::ops::Deref;
use std::sync::Arc;

use crate::activitystream::ObjectType;
use crate::activitystream::ToJson;
use crate::activitystream::Activity;
use crate::activitystream::{types::ActivityType, Object, BaseType, LinkedObject};
use crate::model::{activity, object, user};
use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};

pub async fn serve(db: DatabaseConnection) {
	// build our application with a single route
	let app = Router::new()
		.route("/inbox", post(inbox))
		.route("/outbox", get(|| async { todo!() }))
		.route("/users/:id", get(user))
		.route("/activities/:id", get(activity))
		.route("/objects/:id", get(object))
		.with_state(Arc::new(db));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

	axum::serve(listener, app)
		.await
		.unwrap();
}

async fn inbox(State(db) : State<Arc<DatabaseConnection>>, Json(object): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, StatusCode> {
	match object.full_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(BaseType::Link(_x)) => Err(StatusCode::UNPROCESSABLE_ENTITY), // we could but not yet
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => Err(StatusCode::UNPROCESSABLE_ENTITY),
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
			let Ok(activity_entity) = activity::Model::new(&object) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Some(LinkedObject::Object(obj)) = object.object() else {
				// TODO we could process non-embedded activities but im lazy rn
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Ok(obj_entity) = object::Model::new(&obj) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			object::Entity::insert(obj_entity.into_active_model())
				.exec(db.deref())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			activity::Entity::insert(activity_entity.into_active_model())
				.exec(db.deref())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(Json(serde_json::Value::Null)) // TODO hmmmmmmmmmmm not the best value to return....
		},
		Some(BaseType::Object(ObjectType::Activity(_x))) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(_x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}

async fn user(State(db) : State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	let uri = format!("http://localhost:3000/users/{id}");
	match user::Entity::find_by_id(uri).one(db.deref()).await {
		Ok(Some(user)) => Ok(Json(user.json())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for user: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

async fn activity(State(db) : State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	let uri = format!("http://localhost:3000/activities/{id}");
	match activity::Entity::find_by_id(uri).one(db.deref()).await {
		Ok(Some(activity)) => Ok(Json(activity.json())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for activity: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

async fn object(State(db) : State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	let uri = format!("http://localhost:3000/objects/{id}");
	match object::Entity::find_by_id(uri).one(db.deref()).await {
		Ok(Some(object)) => Ok(Json(object.json())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for object: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}
