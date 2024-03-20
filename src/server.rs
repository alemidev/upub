use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use sea_orm::DatabaseConnection;
use crate::activitypub as ap;

pub async fn serve(db: DatabaseConnection) {
	// build our application with a single route
	let app = Router::new()
		// core server inbox/outbox, maybe for feeds? TODO do we need these?
		.route("/inbox", post(ap::inbox))
		.route("/outbox", get(ap::outbox))
		// actor routes
		.route("/users/:id", get(ap::user::view))
		.route("/users/:id/inbox", post(ap::user::inbox))
		.route("/users/:id/outbox", get(ap::user::outbox))
		// specific object routes
		.route("/activities/:id", get(ap::activity::view))
		.route("/objects/:id", get(ap::object::view))
		.with_state(Arc::new(db));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

	axum::serve(listener, app)
		.await
		.unwrap();
}
