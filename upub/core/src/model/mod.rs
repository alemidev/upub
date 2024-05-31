pub mod actor;
pub mod object;
pub mod activity;

pub mod config;
pub mod credential;
pub mod session;

pub mod instance;
pub mod delivery;

pub mod relation;
pub mod announce;
pub mod like;

pub mod hashtag;
pub mod mention;
pub mod attachment;

pub mod addressing;

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

impl<T: apb::Base> From<apb::Node<T>> for Audience {
	fn from(value: apb::Node<T>) -> Self {
		Audience(
			match value {
				apb::Node::Empty => vec![],
				apb::Node::Link(l) => vec![l.href().to_string()],
				apb::Node::Object(o) => if let Some(id) = o.id() { vec![id.to_string()] } else { vec![] },
				apb::Node::Array(arr) => arr.into_iter().filter_map(|l| Some(l.id()?.to_string())).collect(),
			}
		)
	}
}
