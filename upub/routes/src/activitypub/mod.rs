pub mod actor;
pub mod inbox;
pub mod outbox;
pub mod object;
pub mod activity;
pub mod application;
pub mod auth;
pub mod tags;
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
			.route("/fetch", get(ap::application::ap_fetch))
			.route("/proxy/:hmac/:uri", get(ap::application::cloak_proxy))
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
			.route("/actors/:id", get(ap::actor::view))
			.route("/actors/:id/inbox", post(ap::actor::inbox::post))
			.route("/actors/:id/inbox", get(ap::actor::inbox::get))
			.route("/actors/:id/inbox/page", get(ap::actor::inbox::page))
			.route("/actors/:id/outbox", post(ap::actor::outbox::post))
			.route("/actors/:id/outbox", get(ap::actor::outbox::get))
			.route("/actors/:id/outbox/page", get(ap::actor::outbox::page))
			.route("/actors/:id/notifications", get(ap::actor::notifications::get))
			.route("/actors/:id/notifications/page", get(ap::actor::notifications::page))
			.route("/actors/:id/followers", get(ap::actor::following::get::<false>))
			.route("/actors/:id/followers/page", get(ap::actor::following::page::<false>))
			.route("/actors/:id/following", get(ap::actor::following::get::<true>))
			.route("/actors/:id/following/page", get(ap::actor::following::page::<true>))
			// .route("/actors/:id/audience", get(ap::actor::audience::get))
			// .route("/actors/:id/audience/page", get(ap::actor::audience::page))
			// activities
			.route("/activities/:id", get(ap::activity::view))
			// hashtags
			.route("/tags/:id", get(ap::tags::get))
			.route("/tags/:id/page", get(ap::tags::page))
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
			// hashtags routes
			//.route("/hashtags/:name", get(ap::hashtags::get))
			//.route("/hashtags/:name/page", get(ap::hashtags::page))
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
