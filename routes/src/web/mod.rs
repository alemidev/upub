use axum::{response::IntoResponse, routing, Router, http};

impl super::WebRouter for Router<upub::Context> {
	fn web_routes(self) -> Self {
		self
			.route("/web/assets/upub-web.js", routing::get(upub_web_js))
			.route("/web/assets/upub-web_bg.wasm", routing::get(upub_web_wasm))
			.route("/web/assets/style.css", routing::get(upub_style_css))
			.route("/web/assets/favicon.ico", routing::get(upub_favicon))
			.route("/web/assets/icon.png", routing::get(upub_pwa_icon))
			.route("/web/assets/manifest.json", routing::get(upub_pwa_manifest))
			.route("/web", routing::get(upub_web_index))
			.route("/web/", routing::get(upub_web_index))
			.route("/web/{*any}", routing::get(upub_web_index))
	}
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
