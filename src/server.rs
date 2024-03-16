use std::ops::Deref;
use std::sync::Arc;

use crate::activitystream::object::ToJson;
use crate::activitystream::{types::ActivityType, Object, Type};
use crate::model::user;
use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn serve(db: DatabaseConnection) {
	// build our application with a single route
	let app = Router::new()
		.route("/inbox", post(inbox))
		.route("/outbox", get(|| async { todo!() }))
		.route("/users/:id", get(user))
		.route("/objects/:id", get(object))
		.with_state(Arc::new(db));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

	axum::serve(listener, app)
		.await
		.unwrap();
}

async fn inbox(State(_db) : State<Arc<DatabaseConnection>>, Json(object): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, StatusCode> {
	match object.object_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(Type::Activity) => { Err(StatusCode::UNPROCESSABLE_ENTITY) },
		Some(Type::ActivityType(ActivityType::Follow)) => { todo!() },
		Some(Type::ActivityType(ActivityType::Create)) => { todo!() },
		Some(Type::ActivityType(ActivityType::Like)) => { todo!() },
		Some(Type::ActivityType(_x)) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(_x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}

async fn user(State(db) : State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	match user::Entity::find_by_id(id).one(db.deref()).await {
		Ok(Some(user)) => Ok(Json(user.json())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for user: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

async fn object(State(_db) : State<Arc<DatabaseConnection>>, Path(_id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}
