pub mod activity;
pub mod actor;
pub mod collection;
pub mod document;
pub mod tombstone;
pub mod place;
pub mod profile;
pub mod relationship;

use crate::{getter, setter, strenum};

use super::Node;

use actor::{Actor, ActorType};
use document::{Image, DocumentType};
use activity::ActivityType;
use collection::{Collection, CollectionType};

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

pub trait ObjectMut : super::BaseMut {
	fn set_object_type(&mut self, val: Option<ObjectType>) -> &mut Self;
	fn set_attachment(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_attributed_to(&mut self, val: Node<impl Actor>) -> &mut Self;
	fn set_audience(&mut self, val: Node<impl Actor>) -> &mut Self;
	fn set_content(&mut self, val: Option<&str>) -> &mut Self; // TODO handle language maps
	fn set_context(&mut self, val: Node<impl Object>) -> &mut Self; 
	fn set_name(&mut self, val: Option<&str>) -> &mut Self;       // also in link // TODO handle language maps
	fn set_end_time(&mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> &mut Self;
	fn set_generator(&mut self, val: Node<impl Actor>) -> &mut Self;
	fn set_icon(&mut self, val: Node<impl Image>) -> &mut Self;
	fn set_image(&mut self, val: Node<impl Image>) -> &mut Self;
	fn set_in_reply_to(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_location(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_preview(&mut self, val: Node<impl Object>) -> &mut Self;    // also in link
	fn set_published(&mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> &mut Self;
	fn set_replies(&mut self, val: Node<impl Collection>) -> &mut Self;
	fn set_start_time(&mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> &mut Self;
	fn set_summary(&mut self, val: Option<&str>) -> &mut Self;
	fn set_tag(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_updated(&mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> &mut Self;
	fn set_url(&mut self, val: Option<Vec<impl super::Link>>) -> &mut Self;
	fn set_to(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_bto(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_cc(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_bcc(&mut self, val: Node<impl Object>) -> &mut Self;
	fn set_media_type(&mut self, val: Option<&str>) -> &mut Self; // also in link
	fn set_duration(&mut self, val: Option<&str>) -> &mut Self; // TODO how to parse xsd:duration ?
}

impl Object for serde_json::Value {
	
	getter! { object_type -> type ObjectType }
	getter! { attachment -> node impl Object }
	getter! { attributed_to::attributedTo -> node impl Actor }
	getter! { audience -> node impl Actor }
	getter! { content -> &str }
	getter! { context -> node impl Object }
	getter! { name -> &str }
	getter! { end_time::endTime -> chrono::DateTime<chrono::Utc> }
	getter! { generator -> node impl Actor }
	getter! { icon -> node impl Image }
	getter! { image -> node impl Image }
	getter! { in_reply_to::inReplyTo -> node impl Object }
	getter! { location -> node impl Object }
	getter! { preview -> node impl Object }
	getter! { published -> chrono::DateTime<chrono::Utc> }
	getter! { replies -> node impl Collection }
	getter! { start_time::startTime -> chrono::DateTime<chrono::Utc> }
	getter! { summary -> &str }
	getter! { tag -> node impl Object }
	getter! { updated -> chrono::DateTime<chrono::Utc> }
	getter! { to -> node impl Object }
	getter! { bto -> node impl Object }
	getter! { cc -> node impl Object }
	getter! { bcc -> node impl Object }
	getter! { media_type -> &str }
	getter! { duration -> &str }

	fn url(&self) -> Option<Vec<impl super::Link>> { 
		Some(
			self.get("url")?
				.as_array()?
				.iter()
				.filter_map(|x| Some(x.as_str()?.to_string()))
				.collect()
		)
	}
}

impl ObjectMut for serde_json::Value {
	setter! { object_type -> type ObjectType }
	setter! { attachment -> node impl Object }
	setter! { attributed_to::attributedTo -> node impl Actor }
	setter! { audience -> node impl Actor }
	setter! { content -> &str }
	setter! { context -> node impl Object }
	setter! { name -> &str }
	setter! { end_time::endTime -> chrono::DateTime<chrono::Utc> }
	setter! { generator -> node impl Actor }
	setter! { icon -> node impl Image }
	setter! { image -> node impl Image }
	setter! { in_reply_to::inReplyTo -> node impl Object }
	setter! { location -> node impl Object }
	setter! { preview -> node impl Object }
	setter! { published -> chrono::DateTime<chrono::Utc> }
	setter! { replies -> node impl Collection }
	setter! { start_time::startTime -> chrono::DateTime<chrono::Utc> }
	setter! { summary -> &str }
	setter! { tag -> node impl Object }
	setter! { updated -> chrono::DateTime<chrono::Utc> }
	setter! { to -> node impl Object }
	setter! { bto -> node impl Object}
	setter! { cc -> node impl Object }
	setter! { bcc -> node impl Object }
	setter! { media_type -> &str }
	setter! { duration -> &str }

	fn set_url(&mut self, _val: Option<Vec<impl super::Link>>) -> &mut Self {
		todo!()
	}
}
