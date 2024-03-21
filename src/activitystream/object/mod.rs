pub mod activity;
pub mod actor;
pub mod collection;
pub mod document;
pub mod tombstone;
pub mod place;
pub mod profile;
pub mod relationship;

use crate::{getter, setter, strenum};

use super::{Link, Node};

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
	fn url(&self) -> Node<impl super::Link> { Node::empty() }
	fn to(&self) -> Node<impl Link> { Node::Empty::<serde_json::Value> }
	fn bto(&self) -> Node<impl Link> { Node::Empty::<serde_json::Value> }
	fn cc(&self) -> Node<impl Link> { Node::Empty::<serde_json::Value> }
	fn bcc(&self) -> Node<impl Link> { Node::Empty::<serde_json::Value> }
	fn media_type(&self) -> Option<&str> { None } // also in link
	fn duration(&self) -> Option<&str> { None } // TODO how to parse xsd:duration ?
}

pub trait ObjectMut : super::BaseMut {
	fn set_object_type(self, val: Option<ObjectType>) -> Self;
	fn set_attachment(self, val: Node<impl Object>) -> Self;
	fn set_attributed_to(self, val: Node<impl Actor>) -> Self;
	fn set_audience(self, val: Node<impl Actor>) -> Self;
	fn set_content(self, val: Option<&str>) -> Self; // TODO handle language maps
	fn set_context(self, val: Node<impl Object>) -> Self; 
	fn set_name(self, val: Option<&str>) -> Self;       // also in link // TODO handle language maps
	fn set_end_time(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_generator(self, val: Node<impl Actor>) -> Self;
	fn set_icon(self, val: Node<impl Image>) -> Self;
	fn set_image(self, val: Node<impl Image>) -> Self;
	fn set_in_reply_to(self, val: Node<impl Object>) -> Self;
	fn set_location(self, val: Node<impl Object>) -> Self;
	fn set_preview(self, val: Node<impl Object>) -> Self;    // also in link
	fn set_published(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_replies(self, val: Node<impl Collection>) -> Self;
	fn set_start_time(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_summary(self, val: Option<&str>) -> Self;
	fn set_tag(self, val: Node<impl Object>) -> Self;
	fn set_updated(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_url(self, val: Node<impl super::Link>) -> Self;
	fn set_to(self, val: Node<impl Link>) -> Self;
	fn set_bto(self, val: Node<impl Link>) -> Self;
	fn set_cc(self, val: Node<impl Link>) -> Self;
	fn set_bcc(self, val: Node<impl Link>) -> Self;
	fn set_media_type(self, val: Option<&str>) -> Self; // also in link
	fn set_duration(self, val: Option<&str>) -> Self; // TODO how to parse xsd:duration ?
}

impl Object for serde_json::Value {
	
	getter! { object_type -> type ObjectType }
	getter! { attachment -> node impl Object }
	getter! { attributed_to::attributedTo -> node impl Actor }
	getter! { audience -> node impl Actor }
	getter! { content -> &str }
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
	getter! { to -> node impl Link }
	getter! { bto -> node impl Link }
	getter! { cc -> node impl Link }
	getter! { bcc -> node impl Link }
	getter! { media_type -> &str }
	getter! { duration -> &str }
	getter! { url -> node impl super::Link }

	// TODO Mastodon doesn't use a "context" field on the object but makes up a new one!!
	fn context(&self) -> Node<impl Object> {
		match self.get("context") {
			Some(x) => Node::from(x.clone()),
			None => match self.get("conversation") {
				Some(x) => Node::from(x.clone()),
				None => Node::empty(),
			}
		}
	}
}

impl ObjectMut for serde_json::Value {
	setter! { object_type -> type ObjectType }
	setter! { attachment -> node impl Object }
	setter! { attributed_to::attributedTo -> node impl Actor }
	setter! { audience -> node impl Actor }
	setter! { content -> &str }
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
	setter! { to -> node impl Link }
	setter! { bto -> node impl Link}
	setter! { cc -> node impl Link }
	setter! { bcc -> node impl Link }
	setter! { media_type -> &str }
	setter! { duration -> &str }
	setter! { url -> node impl super::Link }

	// TODO Mastodon doesn't use a "context" field on the object but makes up a new one!!
	fn set_context(mut self, ctx: Node<impl Object>) -> Self {
		if let Some(conversation) = ctx.id() {
			crate::activitystream::macros::set_maybe_value(
				&mut self, "conversation", Some(serde_json::Value::String(conversation.to_string())),
			);
		}
		crate::activitystream::macros::set_maybe_node(
			&mut self, "context", ctx
		);
		self
	}

}
