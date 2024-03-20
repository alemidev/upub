use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use sea_orm::DatabaseConnection;
use crate::activitypub as ap;

#[derive(Clone)]
pub struct Context(Arc<ContextInner>);
struct ContextInner {
	db: DatabaseConnection,
	domain: String,
}
impl Context {
	pub fn new(db: DatabaseConnection, mut domain: String) -> Self {
		if !domain.starts_with("http") {
			domain = format!("https://{domain}");
		}
		if domain.ends_with('/') {
			domain.replace_range(domain.len()-1.., "");
		}
		Context(Arc::new(ContextInner { db, domain }))
	}

	pub fn db(&self) -> &DatabaseConnection {
		&self.0.db
	}

	pub fn uri(&self, entity: &str, id: String) -> String {
		if id.starts_with("http") { id } else {
			format!("{}/{}/{}", self.0.domain, entity, id)
		}
	}

	pub fn id(&self, id: String) -> String {
		if id.starts_with(&self.0.domain) {
			let mut out = id.replace(&self.0.domain, "");
			if out.ends_with('/') {
				out.replace_range(out.len()-1.., "");
			}
			out
		} else {
			id
		}
	}
}

pub async fn serve(db: DatabaseConnection, domain: String) {
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
		.with_state(Context::new(db, domain));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

	axum::serve(listener, app)
		.await
		.unwrap();
}
