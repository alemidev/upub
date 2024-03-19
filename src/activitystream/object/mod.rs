pub mod actor;
pub use actor::{Actor, Profile, ActorType};

pub mod collection;
pub use collection::{Collection, CollectionPage, CollectionType};

pub mod document;
pub use document::{Document, Image, Place, DocumentType};

pub mod activity;
pub use activity::{Activity, ActivityType};

pub mod tombstone;
pub use tombstone::Tombstone;

pub mod relationship;
pub use relationship::Relationship;

use crate::strenum;

use super::{node::NodeExtractor, Node};

strenum! {
	pub enum ObjectType {
		Object,
		Article,
		Event,
		Note,
		Place,
		Profile,
		Relationship,
		Tombstone;
	
		Activity(ActivityType),
		Actor(ActorType),
		Collection(CollectionType),
		Document(DocumentType)
	};
}

pub trait Object : super::Base {
	fn object_type(&self) -> Option<ObjectType> { None }
	fn attachment(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn attributed_to(&self) -> Node<impl Actor> { Node::Empty::<serde_json::Value> }
	fn audience(&self) -> Node<impl Actor> { Node::Empty::<serde_json::Value> }
	fn content(&self) -> Option<&str> { None } // TODO handle language maps
	fn context(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> } 
	fn name(&self) -> Option<&str> { None }       // also in link // TODO handle language maps
	fn end_time(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn generator(&self) -> Node<impl Actor> { Node::Empty::<serde_json::Value> }
	fn icon(&self) -> Node<impl Image> { Node::Empty::<serde_json::Value> }
	fn image(&self) -> Node<impl Image> { Node::Empty::<serde_json::Value> }
	fn in_reply_to(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn location(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn preview(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }    // also in link
	fn published(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn replies(&self) -> Node<impl Collection> { Node::Empty::<serde_json::Value> }
	fn start_time(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn summary(&self) -> Option<&str> { None }
	fn tag(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn updated(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn url(&self) -> Option<Vec<impl super::Link>> { None::<Vec<serde_json::Value>> }
	fn to(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn bto(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn cc(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn bcc(&self) -> Node<impl Object> { Node::Empty::<serde_json::Value> }
	fn media_type(&self) -> Option<&str> { None } // also in link
	fn duration(&self) -> Option<&str> { None } // TODO how to parse xsd:duration ?
}

impl Object for serde_json::Value {
	fn object_type(&self) -> Option<ObjectType> {
		use super::Base;
		match self.base_type() {
			Some(super::BaseType::Object(o)) => Some(o),
			_ => None,
		}
	}

	fn attachment(&self) -> Node<impl Object> {
		self.node_vec("attachment")
	}

	fn attributed_to(&self) -> Node<impl Actor> {
		self.node_vec("attributedTo")
	}

	fn audience(&self) -> Node<impl Actor> {
		self.node_vec("audience")
	}

	fn content(&self) -> Option<&str> {
		self.get("content")?.as_str()
	}

	fn name(&self) -> Option<&str> {
		self.get("name")?.as_str()
	}

	fn end_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
		Some(
			chrono::DateTime::parse_from_rfc3339(
					self
						.get("endTime")?
						.as_str()?
				)
				.ok()?
				.with_timezone(&chrono::Utc))
	}

	fn generator(&self) -> Node<impl Actor> {
		self.node_vec("generator")
	}

	fn icon(&self) -> Node<impl Image> {
		self.node_vec("icon")
	}

	fn image(&self) -> Node<impl Image> {
		self.node_vec("image")
	}

	fn in_reply_to(&self) -> Node<impl Object> {
		self.node_vec("inReplyTo")
	}

	fn location(&self) -> Node<impl Object> {
		self.node_vec("location")
	}
}
