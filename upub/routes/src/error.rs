use axum::{http::StatusCode, response::Redirect};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("database error: {0:?}")]
	Database(#[from] sea_orm::DbErr),

	#[error("encountered malformed object: {0}")]
	Field(#[from] apb::FieldErr),

	#[error("http signature error: {0:?}")]
	HttpSignature(#[from] httpsign::HttpSignatureError),

	#[error("outgoing request error: {0:?}")]
	Reqwest(#[from] reqwest::Error),

	// TODO this is quite ugly because its basically a reqwest::Error but with extra string... buuut
	// helps with debugging!
	#[error("fetch error: {0:?}")]
	FetchError(#[from] upub::traits::fetch::RequestError),

	// wrapper error to return arbitraty status codes
	#[error("{0}")]
	Status(StatusCode),

	// TODO this isn't really an error but i need to redirect from some routes so this allows me to
	// keep the type hints on the return type, still what the hell!!!!
	#[error("redirecting to {0}")]
	Redirect(String),
}

impl ApiError {
	pub fn bad_request() -> Self {
		Self::Status(axum::http::StatusCode::BAD_REQUEST)
	}

	pub fn unprocessable() -> Self {
		Self::Status(axum::http::StatusCode::UNPROCESSABLE_ENTITY)
	}

	pub fn not_found() -> Self {
		Self::Status(axum::http::StatusCode::NOT_FOUND)
	}

	pub fn forbidden() -> Self {
		Self::Status(axum::http::StatusCode::FORBIDDEN)
	}

	pub fn unauthorized() -> Self {
		Self::Status(axum::http::StatusCode::UNAUTHORIZED)
	}

	pub fn not_modified() -> Self {
		Self::Status(axum::http::StatusCode::NOT_MODIFIED)
	}

	pub fn internal_server_error() -> Self {
		Self::Status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
	}
}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<axum::http::StatusCode> for ApiError {
	fn from(value: axum::http::StatusCode) -> Self {
		ApiError::Status(value)
	}
}

impl axum::response::IntoResponse for ApiError {
	fn into_response(self) -> axum::response::Response {
		// TODO it's kind of jank to hide this print down here, i should probably learn how spans work
		//      in tracing and use the library's features but ehhhh
		tracing::debug!("emitting error response: {self:?}");
		let descr = self.to_string();
		match self {
			ApiError::Redirect(to) => Redirect::to(&to).into_response(),
			ApiError::Status(status) => status.into_response(),
			ApiError::Database(e) => (
				StatusCode::SERVICE_UNAVAILABLE,
				axum::Json(serde_json::json!({
					"error": "database",
					"inner": format!("{e:#?}"),
				}))
			).into_response(),
			ApiError::Reqwest(x) => (
				x.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
				axum::Json(serde_json::json!({
					"error": "request",
					"status": x.status().map(|s| s.to_string()).unwrap_or_default(),
					"url": x.url().map(|x| x.to_string()).unwrap_or_default(),
					"description": descr,
					"inner": format!("{x:#?}"),
				}))
			).into_response(),
			ApiError::FetchError(pull) => (
				StatusCode::INTERNAL_SERVER_ERROR,
				axum::Json(serde_json::json!({
					"error": "fetch",
					"description": descr,
					"inner": format!("{pull:#?}"),
				}))
			).into_response(),
			ApiError::Field(x) => (
				axum::http::StatusCode::BAD_REQUEST,
				axum::Json(serde_json::json!({
					"error": "field",
					"field": x.0.to_string(),
					"description": descr,
				}))
			).into_response(),
			x => (
				StatusCode::INTERNAL_SERVER_ERROR,
				axum::Json(serde_json::json!({
					"error": "unknown",
					"description": descr,
					"inner": format!("{x:#?}"),
				}))
			).into_response(),
		}
	}
}
