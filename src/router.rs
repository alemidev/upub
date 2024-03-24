use axum::{routing::{get, post}, Router};
use sea_orm::DatabaseConnection;
use crate::activitypub as ap;

pub async fn serve(db: DatabaseConnection, domain: String) {
	// build our application with a single route
	let app = Router::new()
		// core server inbox/outbox, maybe for feeds? TODO do we need these?
		.route("/", get(ap::view))
		// TODO shared inboxes and instance stream will come later, just use users *boxes for now
		.route("/inbox", get(ap::inbox::get))
		// .route("/inbox", post(ap::inbox::post))
		// .route("/outbox", get(ap::outbox::get))
		// .route("/outbox", get(ap::outbox::post))
		// AUTH routes
		.route("/auth", post(ap::auth))
		// .well-known and discovery
		.route("/.well-known/webfinger", get(ap::well_known::webfinger))
		.route("/.well-known/host-meta", get(ap::well_known::host_meta))
		.route("/.well-known/nodeinfo", get(ap::well_known::nodeinfo_discovery))
		.route("/nodeinfo/:version", get(ap::well_known::nodeinfo))
		// actor routes
		.route("/users/:id", get(ap::user::view))
		.route("/users/:id/inbox", post(ap::user::inbox))
		.route("/users/:id/outbox", get(ap::user::outbox))
		.route("/users/:id/followers", get(ap::user::follow___::<false>))
		.route("/users/:id/following", get(ap::user::follow___::<true>))
		// specific object routes
		.route("/activities/:id", get(ap::activity::view))
		.route("/objects/:id", get(ap::object::view))
		.with_state(crate::server::Context::new(db, domain));

	// run our app with hyper, listening locally on port 3000
	let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

	axum::serve(listener, app)
		.await
		.unwrap();
}
