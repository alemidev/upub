pub mod activity;
pub mod actor;
pub mod collection;
pub mod document;
pub mod tombstone;
pub mod place;
pub mod profile;
pub mod relationship;

use crate::{getter, setter, strenum};

use super::{Base, BaseMut, Link, Node};

use actor::{Actor, ActorType};
use document::{Document, DocumentType};
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

pub trait Object : Base {
	type Link : Link;
	type Actor : Actor;
	type Object : Object;
	type Collection : Collection;
	type Document : Document;

	fn object_type(&self) -> Option<ObjectType> { None }
	fn attachment(&self) -> Node<Self::Object> { Node::Empty }
	fn attributed_to(&self) -> Node<Self::Actor> { Node::Empty }
	fn audience(&self) -> Node<Self::Actor> { Node::Empty }
	fn content(&self) -> Option<&str> { None } // TODO handle language maps
	fn context(&self) -> Node<Self::Object> { Node::Empty } 
	fn name(&self) -> Option<&str> { None }       // also in link // TODO handle language maps
	fn end_time(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn generator(&self) -> Node<Self::Actor> { Node::Empty }
	fn icon(&self) -> Node<Self::Document> { Node::Empty }
	fn image(&self) -> Node<Self::Document> { Node::Empty }
	fn in_reply_to(&self) -> Node<Self::Object> { Node::Empty }
	fn location(&self) -> Node<Self::Object> { Node::Empty }
	fn preview(&self) -> Node<Self::Object> { Node::Empty }    // also in link
	fn published(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn replies(&self) -> Node<Self::Collection> { Node::Empty }
	fn start_time(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn summary(&self) -> Option<&str> { None }
	fn tag(&self) -> Node<Self::Object> { Node::Empty }
	fn updated(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn url(&self) -> Node<Self::Link> { Node::Empty }
	fn to(&self) -> Node<Self::Link> { Node::Empty }
	fn bto(&self) -> Node<Self::Link> { Node::Empty }
	fn cc(&self) -> Node<Self::Link> { Node::Empty }
	fn bcc(&self) -> Node<Self::Link> { Node::Empty }
	fn media_type(&self) -> Option<&str> { None } // also in link
	fn duration(&self) -> Option<&str> { None } // TODO how to parse xsd:duration ?
}

pub trait ObjectMut : BaseMut {
	type Link : Link;
	type Actor : Actor;
	type Object : Object;
	type Collection : Collection;
	type Document : Document;

	fn set_object_type(self, val: Option<ObjectType>) -> Self;
	fn set_attachment(self, val: Node<Self::Object>) -> Self;
	fn set_attributed_to(self, val: Node<Self::Actor>) -> Self;
	fn set_audience(self, val: Node<Self::Actor>) -> Self;
	fn set_content(self, val: Option<&str>) -> Self; // TODO handle language maps
	fn set_context(self, val: Node<Self::Object>) -> Self; 
	fn set_name(self, val: Option<&str>) -> Self;       // also in link // TODO handle language maps
	fn set_end_time(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_generator(self, val: Node<Self::Actor>) -> Self;
	fn set_icon(self, val: Node<Self::Document>) -> Self;
	fn set_image(self, val: Node<Self::Document>) -> Self;
	fn set_in_reply_to(self, val: Node<Self::Object>) -> Self;
	fn set_location(self, val: Node<Self::Object>) -> Self;
	fn set_preview(self, val: Node<Self::Object>) -> Self;    // also in link
	fn set_published(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_replies(self, val: Node<Self::Collection>) -> Self;
	fn set_start_time(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_summary(self, val: Option<&str>) -> Self;
	fn set_tag(self, val: Node<Self::Object>) -> Self;
	fn set_updated(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_url(self, val: Node<Self::Link>) -> Self;
	fn set_to(self, val: Node<Self::Link>) -> Self;
	fn set_bto(self, val: Node<Self::Link>) -> Self;
	fn set_cc(self, val: Node<Self::Link>) -> Self;
	fn set_bcc(self, val: Node<Self::Link>) -> Self;
	fn set_media_type(self, val: Option<&str>) -> Self; // also in link
	fn set_duration(self, val: Option<&str>) -> Self; // TODO how to parse xsd:duration ?
}

impl Object for serde_json::Value {
	type Link = serde_json::Value;
	type Actor = serde_json::Value;
	type Object = serde_json::Value;
	type Document = serde_json::Value;
	type Collection = serde_json::Value;
	
	getter! { object_type -> type ObjectType }
	getter! { attachment -> node <Self as Object>::Object }
	getter! { attributed_to::attributedTo -> node Self::Actor }
	getter! { audience -> node Self::Actor }
	getter! { content -> &str }
	getter! { name -> &str }
	getter! { end_time::endTime -> chrono::DateTime<chrono::Utc> }
	getter! { generator -> node Self::Actor }
	getter! { icon -> node Self::Document }
	getter! { image -> node Self::Document }
	getter! { in_reply_to::inReplyTo -> node <Self as Object>::Object }
	getter! { location -> node <Self as Object>::Object }
	getter! { preview -> node <Self as Object>::Object }
	getter! { published -> chrono::DateTime<chrono::Utc> }
	getter! { replies -> node Self::Collection }
	getter! { start_time::startTime -> chrono::DateTime<chrono::Utc> }
	getter! { summary -> &str }
	getter! { tag -> node <Self as Object>::Object }
	getter! { updated -> chrono::DateTime<chrono::Utc> }
	getter! { to -> node Self::Link }
	getter! { bto -> node Self::Link }
	getter! { cc -> node Self::Link }
	getter! { bcc -> node Self::Link }
	getter! { media_type -> &str }
	getter! { duration -> &str }
	getter! { url -> node Self::Link }

	// TODO Mastodon doesn't use a "context" field on the object but makes up a new one!!
	fn context(&self) -> Node<<Self as Object>::Object> {
		match self.get("context") {
			Some(x) => Node::from(x.clone()),
			None => match self.get("conversation") {
				Some(x) => Node::from(x.clone()),
				None => Node::Empty,
			}
		}
	}
}

impl ObjectMut for serde_json::Value {
	type Link = serde_json::Value;
	type Actor = serde_json::Value;
	type Object = serde_json::Value;
	type Document = serde_json::Value;
	type Collection = serde_json::Value;

	setter! { object_type -> type ObjectType }
	setter! { attachment -> node <Self as Object>::Object }
	setter! { attributed_to::attributedTo -> node Self::Actor }
	setter! { audience -> node Self::Actor }
	setter! { content -> &str }
	setter! { name -> &str }
	setter! { end_time::endTime -> chrono::DateTime<chrono::Utc> }
	setter! { generator -> node Self::Actor }
	setter! { icon -> node Self::Document }
	setter! { image -> node Self::Document }
	setter! { in_reply_to::inReplyTo -> node <Self as Object>::Object }
	setter! { location -> node <Self as Object>::Object }
	setter! { preview -> node <Self as Object>::Object }
	setter! { published -> chrono::DateTime<chrono::Utc> }
	setter! { replies -> node Self::Collection }
	setter! { start_time::startTime -> chrono::DateTime<chrono::Utc> }
	setter! { summary -> &str }
	setter! { tag -> node <Self as Object>::Object }
	setter! { updated -> chrono::DateTime<chrono::Utc> }
	setter! { to -> node Self::Link }
	setter! { bto -> node Self::Link}
	setter! { cc -> node Self::Link }
	setter! { bcc -> node Self::Link }
	setter! { media_type -> &str }
	setter! { duration -> &str }
	setter! { url -> node Self::Link }

	// TODO Mastodon doesn't use a "context" field on the object but makes up a new one!!
	fn set_context(mut self, ctx: Node<<Self as Object>::Object>) -> Self {
		if let Some(conversation) = ctx.id() {
			crate::macros::set_maybe_value(
				&mut self, "conversation", Some(serde_json::Value::String(conversation)),
			);
		}
		crate::macros::set_maybe_node(
			&mut self, "context", ctx
		);
		self
	}

}
