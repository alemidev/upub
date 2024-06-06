pub mod actor;
pub mod object;
pub mod activity;

pub mod config;
pub mod credential;
pub mod session;

pub mod instance;
pub mod addressing;
pub mod job;

pub mod relation;
pub mod announce;
pub mod like;

pub mod hashtag;
pub mod mention;
pub mod attachment;


#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, sea_orm::FromJsonQueryResult)]
pub struct Audience(pub Vec<String>);

impl<T: apb::Base> From<apb::Node<T>> for Audience {
	fn from(value: apb::Node<T>) -> Self {
		use apb::field::OptionalString;

		Audience(
			match value {
				apb::Node::Empty => vec![],
				apb::Node::Link(l) => vec![l.href().to_string()],
				apb::Node::Object(o) => if let Ok(id) = o.id() { vec![id.to_string()] } else { vec![] },
				apb::Node::Array(arr) => arr.into_iter().filter_map(|l| l.id().str()).collect(),
			}
		)
	}
}

