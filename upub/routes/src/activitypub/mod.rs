pub mod user;
pub mod inbox;
pub mod outbox;
pub mod object;
pub mod activity;
pub mod application;
pub mod auth;
pub mod well_known;

use axum::{http::StatusCode, response::IntoResponse, routing::{get, patch, post, put}, Router};

pub trait ActivityPubRouter {
	fn ap_routes(self) -> Self;
}

impl ActivityPubRouter for Router<upub::Context> {
	fn ap_routes(self) -> Self {
		use crate::activitypub as ap; // TODO use self ?
	
		self
			// core server inbox/outbox, maybe for feeds? TODO do we need these?
			.route("/", get(ap::application::view))
			// fetch route, to debug and retreive remote objects
			.route("/proxy", post(ap::application::proxy_form))
			.route("/proxy", get(ap::application::proxy_get))
			.route("/proxy/:uri", get(ap::application::proxy_path))
			// TODO shared inboxes and instance stream will come later, just use users *boxes for now
			.route("/inbox", post(ap::inbox::post))
			.route("/inbox", get(ap::inbox::get))
			.route("/inbox/page", get(ap::inbox::page))
			.route("/outbox", post(ap::outbox::post))
			.route("/outbox", get(ap::outbox::get))
			.route("/outbox/page", get(ap::outbox::page))
			// AUTH routes
			.route("/auth", put(ap::auth::register))
			.route("/auth", post(ap::auth::login))
			.route("/auth", patch(ap::auth::refresh))
			// .well-known and discovery
			.route("/.well-known/webfinger", get(ap::well_known::webfinger))
			.route("/.well-known/host-meta", get(ap::well_known::host_meta))
			.route("/.well-known/nodeinfo", get(ap::well_known::nodeinfo_discovery))
			.route("/.well-known/oauth-authorization-server", get(ap::well_known::oauth_authorization_server))
			.route("/nodeinfo/:version", get(ap::well_known::nodeinfo))
			// actor routes
			.route("/actors/:id", get(ap::user::view))
			.route("/actors/:id/inbox", post(ap::user::inbox::post))
			.route("/actors/:id/inbox", get(ap::user::inbox::get))
			.route("/actors/:id/inbox/page", get(ap::user::inbox::page))
			.route("/actors/:id/outbox", post(ap::user::outbox::post))
			.route("/actors/:id/outbox", get(ap::user::outbox::get))
			.route("/actors/:id/outbox/page", get(ap::user::outbox::page))
			.route("/actors/:id/followers", get(ap::user::following::get::<false>))
			.route("/actors/:id/followers/page", get(ap::user::following::page::<false>))
			.route("/actors/:id/following", get(ap::user::following::get::<true>))
			.route("/actors/:id/following/page", get(ap::user::following::page::<true>))
			// activities
			.route("/activities/:id", get(ap::activity::view))
			// specific object routes
			.route("/objects/:id", get(ap::object::view))
			.route("/objects/:id/replies", get(ap::object::replies::get))
			.route("/objects/:id/replies/page", get(ap::object::replies::page))
			.route("/objects/:id/context", get(ap::object::context::get))
			.route("/objects/:id/context/page", get(ap::object::context::page))
			//.route("/objects/:id/likes", get(ap::object::likes::get))
			//.route("/objects/:id/likes/page", get(ap::object::likes::page))
			//.route("/objects/:id/shares", get(ap::object::announces::get))
			//.route("/objects/:id/shares/page", get(ap::object::announces::page))
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct TryFetch {
	#[serde(default)]
	pub fetch: bool,
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
