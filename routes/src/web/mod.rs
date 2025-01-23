use axum::{response::IntoResponse, routing, Router, http};

pub fn web_routes(ctx: upub::Context) -> Router {
	Router::new()
		.nest("/web", Router::new()
			.nest("/assets", Router::new()
				.route("/upub-web.js", routing::get(upub_web_js))
				.route("/upub-web_bg.wasm", routing::get(upub_web_wasm))
				.route("/style.css", routing::get(upub_style_css))
				.route("/favicon.ico", routing::get(upub_favicon))
				.route("/icon.png", routing::get(upub_pwa_icon))
				.route("/manifest.json", routing::get(upub_pwa_manifest))
			)
			.route("", routing::get(upub_web_index))
			.route("/", routing::get(upub_web_index))
			.route("/{*any}", routing::get(upub_web_index))
		)
		.route_layer(axum::middleware::from_fn(redirect_to_ap))
		.with_state(ctx)
}

async fn redirect_to_ap(
	request: axum::extract::Request,
	next: axum::middleware::Next,
) -> axum::response::Response {

	#[cfg(any(feature = "activitypub", feature = "activitypub-redirect"))]
	{
		let accepts_activity_pub = request.headers()
			.get_all(axum::http::header::CONTENT_TYPE)
			.iter()
			.any(|x|
				x.to_str().map_or(false, |x| apb::jsonld::is_activity_pub_content_type(x))
			);

		let accepts_html = request.headers()
			.get_all(axum::http::header::CONTENT_TYPE)
			.iter()
			.any(|x|
				x.to_str().map_or(false, |x| x.starts_with("text/html"))
			);

		if !accepts_html && accepts_activity_pub {
			let new_uri = request.uri().to_string().replacen("/web", "", 1);
			return axum::response::Redirect::temporary(&new_uri).into_response();
		}
	}

	next.run(request).await
}

async fn upub_web_wasm() -> impl IntoResponse {
	(
		[(http::header::CONTENT_TYPE, "application/wasm")],
		include_bytes!(std::env!("CARGO_UPUB_FRONTEND_WASM"))
	)
}

async fn upub_web_js() -> impl IntoResponse {
	(
		[(http::header::CONTENT_TYPE, "text/javascript")],
		include_str!(std::env!("CARGO_UPUB_FRONTEND_JS"))
	)
}

async fn upub_style_css() -> impl IntoResponse {
	(
		[(http::header::CONTENT_TYPE, "text/css")],
		include_str!(std::env!("CARGO_UPUB_FRONTEND_STYLE"))
	)
}

async fn upub_web_index() -> impl IntoResponse {
	axum::response::Html(
		include_str!(std::env!("CARGO_UPUB_FRONTEND_INDEX"))
	)
}

async fn upub_favicon() -> impl IntoResponse {
	(
		[(http::header::CONTENT_TYPE, "image/x-icon")],
		include_bytes!(std::env!("CARGO_UPUB_FRONTEND_FAVICON"))
	)
}

async fn upub_pwa_icon() -> impl IntoResponse {
	(
		[(http::header::CONTENT_TYPE, "image/png")],
		include_bytes!(std::env!("CARGO_UPUB_FRONTEND_PWA_ICON"))
	)
}

async fn upub_pwa_manifest() -> impl IntoResponse {
	(
		[(http::header::CONTENT_TYPE, "application/json")],
		include_bytes!(std::env!("CARGO_UPUB_FRONTEND_PWA_MANIFEST"))
	)
}
