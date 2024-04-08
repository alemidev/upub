#[derive(Debug, thiserror::Error)]
pub enum UpubError {
	#[error("database error: {0}")]
	Database(#[from] sea_orm::DbErr),

	#[error("api returned {0}")]
	Status(axum::http::StatusCode),

	#[error("missing field: {0}")]
	Field(#[from] crate::model::FieldError),

	#[error("openssl error: {0}")]
	OpenSSL(#[from] openssl::error::ErrorStack),

	#[error("fetch error: {0}")]
	Reqwest(#[from] reqwest::Error),
}

impl UpubError {
	pub fn bad_request() -> Self {
		Self::Status(axum::http::StatusCode::BAD_REQUEST)
	}

	pub fn unprocessable() -> Self {
		Self::Status(axum::http::StatusCode::UNPROCESSABLE_ENTITY)
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
		(
			axum::http::StatusCode::INTERNAL_SERVER_ERROR,
			self.to_string()
		).into_response()
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
