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

#[macro_export]
macro_rules! url {
	($ctx:expr, $($args: tt)*) => {
		format!("{}{}", $ctx.base(), format!($($args)*))
	};
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

	pub fn base(&self) -> &str {
		&self.0.domain
	}

	pub fn uri(&self, entity: &str, id: String) -> String {
		if id.starts_with("http") { id } else {
			format!("{}/{}/{}", self.0.domain, entity, id)
		}
	}

	// TODO maybe redo these with enums? idk

	/// get full user id uri
	pub fn uid(&self, id: String) -> String {
		self.uri("users", id)
	}

	/// get full object id uri
	pub fn oid(&self, id: String) -> String {
		self.uri("objects", id)
	}

	/// get full activity id uri
	pub fn aid(&self, id: String) -> String {
		self.uri("activities", id)
	}

	/// get bare uri, usually an uuid but unspecified
	pub fn id(&self, id: String) -> String {
		if id.starts_with(&self.0.domain) {
			id.split('/').last().unwrap_or("").to_string()
		} else {
			id
		}
	}
}

pub async fn serve(db: DatabaseConnection, domain: String) {
	// build our application with a single route
	let app = Router::new()
		// core server inbox/outbox, maybe for feeds? TODO do we need these?
		.route("/", get(ap::view))
		// TODO shared inboxes and instance stream will come later, just use users *boxes for now
		// .route("/inbox", post(ap::inbox))
		// .route("/outbox", get(ap::outbox))
		// .well-known and discovery
		.route("/.well-known/webfinger", get(ap::well_known::webfinger))
		.route("/.well-known/host-meta", get(ap::well_known::host_meta))
		.route("/.well-known/nodeinfo", get(ap::well_known::nodeinfo_discovery))
		.route("/nodeinfo/:version", get(ap::well_known::nodeinfo))
		// actor routes
		.route("/users/:id", get(ap::user::view))
		.route("/users/:id/inbox", post(ap::user::inbox))
		.route("/users/:id/outbox", get(ap::user::outbox))
		.route("/users/:id/followers", get(ap::user::followers))
		.route("/users/:id/following", get(ap::user::following))
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
