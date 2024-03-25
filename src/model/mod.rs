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

use crate::activitystream::{Link, Node};
impl<T : Link> From<Node<T>> for Audience {
	fn from(value: Node<T>) -> Self {
		Audience(
			match value {
				Node::Empty => vec![],
				Node::Link(l) => vec![l.href().to_string()],
				Node::Object(o) => if let Some(id) = o.id() { vec![id.to_string()] } else { vec![] },
				Node::Array(arr) => arr.into_iter().filter_map(|l| Some(l.id()?.to_string())).collect(),
			}
		)
	}
}

