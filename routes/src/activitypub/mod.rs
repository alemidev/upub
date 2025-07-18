pub mod actor;
pub mod inbox;
pub mod outbox;
pub mod object;
pub mod groups;
pub mod lists;
pub mod threads;
pub mod activity;
pub mod application;
pub mod auth;
pub mod tags;
pub mod file;
pub mod well_known;

use axum::{http::StatusCode, response::IntoResponse, routing::{get, patch, post, put}, Router};

pub fn ap_routes(ctx: upub::Context) -> Router {
	use crate::activitypub as ap; // TODO use self ?

	Router::new()
		.route("/", get(ap::application::view))
		.route("/search/objects", get(ap::application::search_objects))
		.route("/search/actors", get(ap::application::search_actors))
		.route("/search/tags", get(ap::application::search_tags))
		.route("/fetch", get(ap::application::ap_fetch))
		.route("/proxy/{hmac}/{uri}", get(ap::application::cloak_proxy))
		.route("/emoji/{domain}/{name}", get(ap::application::emoji_proxy))
		.route("/inbox", post(ap::inbox::post))
		.route("/inbox", get(ap::inbox::get))
		.route("/inbox/page", get(ap::inbox::page))
		.route("/outbox", post(ap::outbox::post))
		.route("/outbox", get(ap::outbox::get))
		.route("/outbox/page", get(ap::outbox::page))
		.route("/threads", get(ap::threads::get))
		.route("/threads/page", get(ap::threads::page))
		.route("/auth", put(ap::auth::register))
		.route("/auth", post(ap::auth::login))
		.route("/auth", patch(ap::auth::refresh))
		.nest("/.well-known", Router::new()
			.route("/webfinger", get(ap::well_known::webfinger))
			.route("/host-meta", get(ap::well_known::host_meta))
			.route("/nodeinfo", get(ap::well_known::nodeinfo_discovery))
			.route("/oauth-authorization-server", get(ap::well_known::oauth_authorization_server))
		)
		.route("/manifest.json", get(ap::well_known::manifest))
		.route("/nodeinfo/{version}", get(ap::well_known::nodeinfo))
		.route("/groups", get(ap::groups::get))
		.route("/groups/page", get(ap::groups::page))
		.route("/lists/{id}", get(ap::lists::get))
		.route("/lists/{id}/page", get(ap::lists::page))
		.route("/lists/{id}/feed", get(ap::lists::feed))
		.route("/lists/{id}/feed/page", get(ap::lists::feed_page))
		.nest("/actors/{id}", Router::new()
			.route("/", get(ap::actor::view))
			.route("/inbox", post(ap::actor::inbox::post))
			.route("/inbox", get(ap::actor::inbox::get))
			.route("/inbox/page", get(ap::actor::inbox::page))
			.route("/outbox", post(ap::actor::outbox::post))
			.route("/outbox", get(ap::actor::outbox::get))
			.route("/outbox/page", get(ap::actor::outbox::page))
			.route("/notifications", get(ap::actor::notifications::get))
			.route("/notifications/page", get(ap::actor::notifications::page))
			.route("/followers", get(ap::actor::following::get::<false>))
			.route("/followers/page", get(ap::actor::following::page::<false>))
			.route("/following", get(ap::actor::following::get::<true>))
			.route("/following/page", get(ap::actor::following::page::<true>))
			.route("/groups", get(ap::actor::groups::get))
			.route("/groups/page", get(ap::actor::groups::page))
			.route("/lists", get(ap::actor::lists::get))
			.route("/lists/page", get(ap::actor::lists::page))
			// .route("/audience", get(ap::actor::audience::get))
			// .route("/audience/page", get(ap::actor::audience::page))
			.route("/likes", get(ap::actor::likes::get))
			.route("/likes/page", get(ap::actor::likes::page))
		)
		.route("/activities/{id}", get(ap::activity::view))
		.nest("/objects/{id}", Router::new()
			.route("/", get(ap::object::view))
			.route("/replies", get(ap::object::replies::get))
			.route("/replies/page", get(ap::object::replies::page))
			.route("/context", get(ap::object::context::get))
			.route("/context/page", get(ap::object::context::page))
			.route("/likes", get(ap::object::likes::get))
			.route("/likes/page", get(ap::object::likes::page))
			.route("/shares", get(ap::object::shares::get))
			.route("/shares/page", get(ap::object::shares::page))
		)
		.route("/tags/{id}", get(ap::tags::get))
		.route("/tags/{id}/page", get(ap::tags::page))
		.route("/file", post(ap::file::upload))
		.route("/file/{id}", get(ap::file::download))
		.route_layer(axum::middleware::from_fn(redirect_to_web))
		.with_state(ctx)
}

async fn redirect_to_web(
	request: axum::extract::Request,
	next: axum::middleware::Next,
) -> axum::response::Response {

	#[cfg(any(feature = "web", feature = "web-redirect"))]
	{
		let (accepts_activity_pub, accepts_html) = crate::builders::accepts_activitypub_html(request.headers());
		if !accepts_activity_pub && accepts_html {
			let uri = request.uri().clone();
			let path_and_query = uri.path_and_query().map(|x| x.as_str()).unwrap_or_default();
			if path_and_query == "/"
				|| path_and_query.starts_with("/objects")
				|| path_and_query.starts_with("/tags")
				|| path_and_query.starts_with("/actors")
			{
				let new_uri = format!(
					"{}{}/web{}",
					uri.scheme().map(|x| format!("{}://", x.as_str())).unwrap_or_default(),
					uri.authority().map(|x| x.as_str()).unwrap_or_default(),
					path_and_query,
				);
				return axum::response::Redirect::temporary(&new_uri).into_response();
			}
		}
	}

	next.run(request).await
}

#[derive(Debug, serde::Deserialize)]
pub struct TryFetch {
	#[serde(default)]
	pub fetch: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
// TODO i don't really like how pleroma/mastodon do it actually, maybe change this?
pub struct Pagination {
	pub offset: Option<u64>,
	pub batch: Option<u64>,
	pub replies: Option<bool>,
}

impl Pagination {
	/// returns (limit, offset), forcing defaults and boundaries
	pub fn pagination(&self) -> (u64, u64) {
		let limit = self.batch.unwrap_or(20).min(50);
		let offset = self.offset.unwrap_or(0);
		(limit, offset)
	}

	pub fn replies(&self) -> bool {
		self.replies.unwrap_or(true)
	}
}

#[derive(Debug, serde::Deserialize)]
// TODO i don't really like how pleroma/mastodon do it actually, maybe change this?
pub struct PaginatedSearch {
	pub q: String,
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
