pub mod user;
pub mod inbox;
pub mod outbox;
pub mod object;
pub mod activity;
pub mod application;
pub mod auth;
pub mod well_known;

pub mod jsonld;
pub use jsonld::JsonLD;

use axum::{http::StatusCode, response::IntoResponse, routing::{get, post}, Router};

pub trait ActivityPubRouter {
	fn ap_routes(self) -> Self;
}

impl ActivityPubRouter for Router<crate::server::Context> {
	fn ap_routes(self) -> Self {
		use crate::routes::activitypub as ap; // TODO use self ?
	
		self
			// core server inbox/outbox, maybe for feeds? TODO do we need these?
			.route("/", get(ap::application::view))
			// TODO shared inboxes and instance stream will come later, just use users *boxes for now
			.route("/inbox", post(ap::inbox::post))
			.route("/inbox", get(ap::inbox::get))
			.route("/inbox/page", get(ap::inbox::page))
			.route("/outbox", post(ap::outbox::post))
			.route("/outbox", get(ap::outbox::get))
			.route("/outbox/page", get(ap::outbox::page))
			// AUTH routes
			.route("/auth", post(ap::auth::login))
			// .well-known and discovery
			.route("/.well-known/webfinger", get(ap::well_known::webfinger))
			.route("/.well-known/host-meta", get(ap::well_known::host_meta))
			.route("/.well-known/nodeinfo", get(ap::well_known::nodeinfo_discovery))
			.route("/nodeinfo/:version", get(ap::well_known::nodeinfo))
			// actor routes
			.route("/users/:id", get(ap::user::view))
			.route("/users/:id/inbox", post(ap::user::inbox::post))
			.route("/users/:id/inbox", get(ap::user::inbox::get))
			.route("/users/:id/inbox/page", get(ap::user::inbox::page))
			.route("/users/:id/outbox", post(ap::user::outbox::post))
			.route("/users/:id/outbox", get(ap::user::outbox::get))
			.route("/users/:id/outbox/page", get(ap::user::outbox::page))
			.route("/users/:id/followers", get(ap::user::following::get::<false>))
			.route("/users/:id/followers/page", get(ap::user::following::page::<false>))
			.route("/users/:id/following", get(ap::user::following::get::<true>))
			.route("/users/:id/following/page", get(ap::user::following::page::<true>))
			// specific object routes
			.route("/activities/:id", get(ap::activity::view))
			.route("/objects/:id", get(ap::object::view))
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct RemoteId {
	pub id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
// TODO i don't really like how pleroma/mastodon do it actually, maybe change this?
pub struct Pagination {
	pub offset: Option<u64>,
	pub batch: Option<u64>,
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
