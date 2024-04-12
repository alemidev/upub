pub mod object;
pub mod activity;
pub mod user;
pub mod config;

pub mod relation;
pub mod addressing;
pub mod share;
pub mod like;
pub mod credential;
pub mod session;
pub mod delivery;
pub mod application;

#[cfg(feature = "faker")]
pub mod faker;

#[derive(Debug, Clone, thiserror::Error)]
#[error("missing required field: '{0}'")]
pub struct FieldError(pub &'static str);

impl From<FieldError> for axum::http::StatusCode {
	fn from(value: FieldError) -> Self {
		tracing::error!("bad request: {value}");
		axum::http::StatusCode::BAD_REQUEST
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, sea_orm::FromJsonQueryResult)]
pub struct Audience(pub Vec<String>);

impl From<apb::Node<serde_json::Value>> for Audience {
	fn from(value: apb::Node<serde_json::Value>) -> Self {
		use apb::{Base, Link};
		Audience(
			match value {
				apb::Node::Empty => vec![],
				apb::Node::Link(l) => vec![l.href().to_string()],
				apb::Node::Object(o) => if let Some(id) = o.id() { vec![id.to_string()] } else { vec![] },
				apb::Node::Array(arr) => arr.into_iter().map(|l| l.href().to_string()).collect(),
			}
		)
	}
}

