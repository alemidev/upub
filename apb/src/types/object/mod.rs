pub mod activity;
pub mod actor;
pub mod collection;
pub mod document;
pub mod tombstone;
pub mod place;
pub mod profile;
pub mod relationship;

use crate::{Base, BaseMut, Node};

use actor::ActorType;
use document::DocumentType;
use activity::ActivityType;
use collection::CollectionType;

crate::strenum! {
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
	type Link : crate::Link;
	type Actor : crate::Actor;
	type Object : Object;
	type Collection : crate::Collection;
	type Document : crate::Document;
	type Activity : crate::Activity;

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
	fn updated(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn replies(&self) -> Node<Self::Collection> { Node::Empty }
	fn likes(&self) -> Node<Self::Collection> { Node::Empty }
	fn shares(&self) -> Node<Self::Collection> { Node::Empty }
	fn start_time(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn summary(&self) -> Option<&str> { None }
	fn tag(&self) -> Node<Self::Object> { Node::Empty }
	fn url(&self) -> Node<Self::Link> { Node::Empty }
	fn to(&self) -> Node<Self::Link> { Node::Empty }
	fn bto(&self) -> Node<Self::Link> { Node::Empty }
	fn cc(&self) -> Node<Self::Link> { Node::Empty }
	fn bcc(&self) -> Node<Self::Link> { Node::Empty }
	fn media_type(&self) -> Option<&str> { None } // also in link
	fn duration(&self) -> Option<&str> { None } // TODO how to parse xsd:duration ?

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn sensitive(&self) -> Option<bool> { None }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn quote_url(&self) -> Node<Self::Object> { Node::Empty }

	#[cfg(feature = "activitypub-fe")]
	fn liked_by_me(&self) -> Option<bool> { None }

	#[cfg(feature = "ostatus")]
	fn conversation(&self) -> Node<Self::Object> { Node::Empty }

	fn as_activity(&self) -> Option<&Self::Activity> { None }
	fn as_actor(&self) -> Option<&Self::Actor> { None }
	fn as_collection(&self) -> Option<&Self::Collection> { None }
	fn as_document(&self) -> Option<&Self::Document> { None }
}

pub trait ObjectMut : BaseMut {
	type Link : crate::Link;
	type Actor : crate::Actor;
	type Object : Object;
	type Collection : crate::Collection;
	type Document : crate::Document;

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
	fn set_updated(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_replies(self, val: Node<Self::Collection>) -> Self;
	fn set_likes(self, val: Node<Self::Collection>) -> Self;
	fn set_shares(self, val: Node<Self::Collection>) -> Self;
	fn set_start_time(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
	fn set_summary(self, val: Option<&str>) -> Self;
	fn set_tag(self, val: Node<Self::Object>) -> Self;
	fn set_url(self, val: Node<Self::Link>) -> Self;
	fn set_to(self, val: Node<Self::Link>) -> Self;
	fn set_bto(self, val: Node<Self::Link>) -> Self;
	fn set_cc(self, val: Node<Self::Link>) -> Self;
	fn set_bcc(self, val: Node<Self::Link>) -> Self;
	fn set_media_type(self, val: Option<&str>) -> Self; // also in link
	fn set_duration(self, val: Option<&str>) -> Self; // TODO how to parse xsd:duration ?

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn set_sensitive(self, val: Option<bool>) -> Self;
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn set_quote_url(self, val: Node<Self::Object>) -> Self;

	#[cfg(feature = "activitypub-fe")]
	fn set_liked_by_me(self, val: Option<bool>) -> Self;

	#[cfg(feature = "ostatus")]
	fn set_conversation(self, val: Node<Self::Object>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Object for serde_json::Value {
	type Link = serde_json::Value;
	type Actor = serde_json::Value;
	type Object = serde_json::Value;
	type Document = serde_json::Value;
	type Collection = serde_json::Value;
	type Activity = serde_json::Value;

	crate::getter! { object_type -> type ObjectType }
	crate::getter! { attachment -> node <Self as Object>::Object }
	crate::getter! { attributed_to::attributedTo -> node Self::Actor }
	crate::getter! { audience -> node Self::Actor }
	crate::getter! { content -> &str }
	crate::getter! { context -> node <Self as Object>::Object }
	crate::getter! { name -> &str }
	crate::getter! { end_time::endTime -> chrono::DateTime<chrono::Utc> }
	crate::getter! { generator -> node Self::Actor }
	crate::getter! { icon -> node Self::Document }
	crate::getter! { image -> node Self::Document }
	crate::getter! { in_reply_to::inReplyTo -> node <Self as Object>::Object }
	crate::getter! { location -> node <Self as Object>::Object }
	crate::getter! { preview -> node <Self as Object>::Object }
	crate::getter! { published -> chrono::DateTime<chrono::Utc> }
	crate::getter! { updated -> chrono::DateTime<chrono::Utc> }
	crate::getter! { replies -> node Self::Collection }
	crate::getter! { likes -> node Self::Collection }
	crate::getter! { shares -> node Self::Collection }
	crate::getter! { start_time::startTime -> chrono::DateTime<chrono::Utc> }
	crate::getter! { summary -> &str }
	crate::getter! { tag -> node <Self as Object>::Object }
	crate::getter! { to -> node Self::Link }
	crate::getter! { bto -> node Self::Link }
	crate::getter! { cc -> node Self::Link }
	crate::getter! { bcc -> node Self::Link }
	crate::getter! { media_type::mediaType -> &str }
	crate::getter! { duration -> &str }
	crate::getter! { url -> node Self::Link }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { sensitive -> bool }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { quote_url::quoteUrl -> node <Self as Object>::Object }

	#[cfg(feature = "activitypub-fe")]
	crate::getter! { liked_by_me::likedByMe -> bool }

	#[cfg(feature = "ostatus")]
	crate::getter! { conversation -> node <Self as Object>::Object }

	fn as_activity(&self) -> Option<&Self::Activity> {
		match self.object_type() {
			Some(ObjectType::Activity(_)) => Some(self),
			_ => None,
		}
	}

	fn as_actor(&self) -> Option<&Self::Actor> {
		match self.object_type() {
			Some(ObjectType::Actor(_)) => Some(self),
			_ => None,
		}
	}

	fn as_collection(&self) -> Option<&Self::Collection> {
		match self.object_type() {
			Some(ObjectType::Collection(_)) => Some(self),
			_ => None,
		}
	}

	fn as_document(&self) -> Option<&Self::Document> {
		match self.object_type() {
			Some(ObjectType::Document(_)) => Some(self),
			_ => None,
		}
	}
}

#[cfg(feature = "unstructured")]
impl ObjectMut for serde_json::Value {
	type Link = serde_json::Value;
	type Actor = serde_json::Value;
	type Object = serde_json::Value;
	type Document = serde_json::Value;
	type Collection = serde_json::Value;

	crate::setter! { object_type -> type ObjectType }
	crate::setter! { attachment -> node <Self as Object>::Object }
	crate::setter! { attributed_to::attributedTo -> node Self::Actor }
	crate::setter! { audience -> node Self::Actor }
	crate::setter! { content -> &str }
	crate::setter! { context -> node <Self as Object>::Object }
	crate::setter! { name -> &str }
	crate::setter! { end_time::endTime -> chrono::DateTime<chrono::Utc> }
	crate::setter! { generator -> node Self::Actor }
	crate::setter! { icon -> node Self::Document }
	crate::setter! { image -> node Self::Document }
	crate::setter! { in_reply_to::inReplyTo -> node <Self as Object>::Object }
	crate::setter! { location -> node <Self as Object>::Object }
	crate::setter! { preview -> node <Self as Object>::Object }
	crate::setter! { published -> chrono::DateTime<chrono::Utc> }
	crate::setter! { updated -> chrono::DateTime<chrono::Utc> }
	crate::setter! { replies -> node Self::Collection }
	crate::setter! { likes -> node Self::Collection }
	crate::setter! { shares -> node Self::Collection }
	crate::setter! { start_time::startTime -> chrono::DateTime<chrono::Utc> }
	crate::setter! { summary -> &str }
	crate::setter! { tag -> node <Self as Object>::Object }
	crate::setter! { to -> node Self::Link }
	crate::setter! { bto -> node Self::Link}
	crate::setter! { cc -> node Self::Link }
	crate::setter! { bcc -> node Self::Link }
	crate::setter! { media_type::mediaType -> &str }
	crate::setter! { duration -> &str }
	crate::setter! { url -> node Self::Link }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { sensitive -> bool }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { quote_url::quoteUrl -> node <Self as Object>::Object }

	#[cfg(feature = "activitypub-fe")]
	crate::setter! { liked_by_me::likedByMe -> bool }

	#[cfg(feature = "ostatus")]
	crate::setter! { conversation -> node <Self as Object>::Object }
}
