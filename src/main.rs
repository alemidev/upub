pub mod model;
pub mod activitystream;
pub mod activitypub;
pub mod server;
pub mod storage;

use activitystream::{types::{ActivityType, ObjectType}, Object, Type};
use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};

#[tokio::main]
async fn main() {
	// build our application with a single route
	let app = Router::new()
		.with_state(())
		.route("/inbox", post(inbox))
		.route("/outbox", get(|| async { todo!() }))
		.route("/users/:id", get(user))
		.route("/objects/:id", get(object));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

	axum::serve(listener, app)
		.await
		.unwrap();
}

async fn inbox(State(ctx) : State<()>, Json(object): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, StatusCode> {
	match object.object_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(Type::Activity) => { Err(StatusCode::UNPROCESSABLE_ENTITY) },
		Some(Type::ActivityType(ActivityType::Follow)) => { todo!() },
		Some(Type::ActivityType(ActivityType::Create)) => { todo!() },
		Some(Type::ActivityType(ActivityType::Like)) => { todo!() },
		Some(Type::ActivityType(x)) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}

async fn user(State(ctx) : State<()>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}

async fn object(State(ctx) : State<()>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}
