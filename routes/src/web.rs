use axum::{response::IntoResponse, routing::get, Router};


impl super::WebRouter for Router<upub::Context> {
	fn web_routes(self) -> Self {
		self
			.route("/web/assets/upub-web.wasm", get(upub_web_wasm))
			.route("/web/assets/style.css", get(upub_style_css))
			.route("/web", get(upub_web_index))
			.route("/web/", get(upub_web_index))
			.route("/web/{*any}", get(upub_web_index))
	}
}



async fn upub_web_wasm() -> impl IntoResponse {
	include_bytes!("../../target/wasm32-unknown-unknown/wasm-release/upub-web.wasm")
}

async fn upub_style_css() -> impl IntoResponse {
	include_str!("../../web/assets/style.css")
}

async fn upub_web_index() -> impl IntoResponse {
	include_str!("../../web/index.html")
}
