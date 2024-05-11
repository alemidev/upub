use axum::{http::StatusCode, response::{IntoResponse, Redirect}};

#[derive(Debug, thiserror::Error)]
pub enum UpubError {
	#[error("database error: {0}")]
	Database(#[from] sea_orm::DbErr),

	#[error("{0}")]
	Status(axum::http::StatusCode),

	#[error("missing field: {0}")]
	Field(#[from] crate::model::FieldError),

	#[error("openssl error: {0}")]
	OpenSSL(#[from] openssl::error::ErrorStack),

	#[error("invalid UTF8 in key: {0}")]
	OpenSSLParse(#[from] std::str::Utf8Error),

	#[error("fetch error: {0}")]
	Reqwest(#[from] reqwest::Error),

	// TODO this is quite ugly because its basically a reqwest::Error but with extra string... buuut
	// helps with debugging!
	#[error("fetch error: {0} -- server responded with {1}")]
	FetchError(reqwest::Error, String),

	#[error("invalid base64 string: {0}")]
	Base64(#[from] base64::DecodeError),

	// TODO this isn't really an error but i need to redirect from some routes so this allows me to
	// keep the type hints on the return type, still what the hell!!!!
	#[error("redirecting to {0}")]
	Redirect(String),
}

impl UpubError {
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

pub type UpubResult<T> = Result<T, UpubError>;

impl From<axum::http::StatusCode> for UpubError {
	fn from(value: axum::http::StatusCode) -> Self {
		UpubError::Status(value)
	}
}

impl axum::response::IntoResponse for UpubError {
	fn into_response(self) -> axum::response::Response {
		let descr = self.to_string();
		match self {
			UpubError::Redirect(to) => Redirect::to(&to).into_response(),
			UpubError::Status(status) => (status, descr).into_response(),
			UpubError::Database(_) => (StatusCode::SERVICE_UNAVAILABLE, descr).into_response(),
			UpubError::FetchError(e, body) =>
				(
					e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
					format!("failed fetching '{}': {descr} -- server responded with {body}", e.url().map(|x| x.to_string()).unwrap_or_default()),
				).into_response(),
			UpubError::Reqwest(x) =>
				(
					x.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
					format!("failed fetching '{}': {descr}", x.url().map(|x| x.to_string()).unwrap_or_default())
				).into_response(),
			UpubError::Field(_) => (axum::http::StatusCode::BAD_REQUEST, descr).into_response(),
			_ => (StatusCode::INTERNAL_SERVER_ERROR, descr).into_response(),
		}
	}
}

pub trait LoggableError {
	fn info_failed(self, msg: &str);
	fn warn_failed(self, msg: &str);
	fn err_failed(self, msg: &str);
}

impl<T, E: std::error::Error> LoggableError for Result<T, E> {
	fn info_failed(self, msg: &str) {
		if let Err(e) = self {
			tracing::info!("{} : {}", msg, e);
		}
	}

	fn warn_failed(self, msg: &str) {
		if let Err(e) = self {
			tracing::warn!("{} : {}", msg, e);
		}
	}

	fn err_failed(self, msg: &str) {
		if let Err(e) = self {
			tracing::error!("{} : {}", msg, e);
		}
	}
}
