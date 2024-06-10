pub mod activity;
pub mod actor;
pub mod collection;
pub mod document;
pub mod tombstone;
pub mod place;
pub mod profile;
pub mod relationship;

use crate::{Base, BaseMut, Field, FieldErr, Node};

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

	fn object_type(&self) -> Field<ObjectType> { Err(FieldErr("type")) }
	/// Identifies a resource attached or related to an object that potentially requires special handling
	/// The intent is to provide a model that is at least semantically similar to attachments in email. 
	fn attachment(&self) -> Node<Self::Object> { Node::Empty }
	/// Identifies one or more entities to which this object is attributed.
	/// The attributed entities might not be Actors. For instance, an object might be attributed to the completion of another activity. 
	fn attributed_to(&self) -> Node<Self::Actor> { Node::Empty }
	/// Identifies one or more entities that represent the total population of entities for which the object can considered to be relevant
	fn audience(&self) -> Node<Self::Object> { Node::Empty }
	/// The content or textual representation of the Object encoded as a JSON string. By default, the value of content is HTML
	/// The mediaType property can be used in the object to indicate a different content type
	/// The content MAY be expressed using multiple language-tagged values
	fn content(&self) -> Field<&str> { Err(FieldErr("content")) } // TODO handle language maps
	/// Identifies the context within which the object exists or an activity was performed
	/// The notion of "context" used is intentionally vague
	/// The intended function is to serve as a means of grouping objects and activities that share a common originating context or purpose
	/// An example could be all activities relating to a common project or event
	fn context(&self) -> Node<Self::Object> { Node::Empty } 
	/// A simple, human-readable, plain-text name for the object. HTML markup MUST NOT be included. The name MAY be expressed using multiple language-tagged values
	fn name(&self) -> Field<&str> { Err(FieldErr("name")) }       // also in link // TODO handle language maps
	/// The date and time describing the actual or expected ending time of the object
	/// When used with an Activity object, for instance, the endTime property specifies the moment the activity concluded or is expected to conclude. 
	fn end_time(&self) -> Field<chrono::DateTime<chrono::Utc>> { Err(FieldErr("endTime")) }
	/// Identifies the entity (e.g. an application) that generated the object
	fn generator(&self) -> Node<Self::Actor> { Node::Empty }
	/// Indicates an entity that describes an icon for this object
	/// The image should have an aspect ratio of one (horizontal) to one (vertical) and should be suitable for presentation at a small size
	fn icon(&self) -> Node<Self::Document> { Node::Empty }
	/// Indicates an entity that describes an image for this object
	/// Unlike the icon property, there are no aspect ratio or display size limitations assumed
	fn image(&self) -> Node<Self::Document> { Node::Empty }
	/// Indicates one or more entities for which this object is considered a response
	fn in_reply_to(&self) -> Node<Self::Object> { Node::Empty }
	/// Indicates one or more physical or logical locations associated with the object
	fn location(&self) -> Node<Self::Object> { Node::Empty }
	/// Identifies an entity that provides a preview of this object
	fn preview(&self) -> Node<Self::Object> { Node::Empty }    // also in link
	/// The date and time at which the object was published
	fn published(&self) -> Field<chrono::DateTime<chrono::Utc>> { Err(FieldErr("published")) }
	/// The date and time at which the object was updated
	fn updated(&self) -> Field<chrono::DateTime<chrono::Utc>> { Err(FieldErr("updated")) }
	/// Identifies a Collection containing objects considered to be responses to this object
	fn replies(&self) -> Node<Self::Collection> { Node::Empty }
	fn likes(&self) -> Node<Self::Collection> { Node::Empty }
	fn shares(&self) -> Node<Self::Collection> { Node::Empty }
	/// The date and time describing the actual or expected starting time of the object.
	/// When used with an Activity object, for instance, the startTime property specifies the moment the activity began or is scheduled to begin. 
	fn start_time(&self) -> Field<chrono::DateTime<chrono::Utc>> { Err(FieldErr("startTime")) }
	/// A natural language summarization of the object encoded as HTML. Multiple language tagged summaries MAY be provided
	fn summary(&self) -> Field<&str> { Err(FieldErr("summary")) }
	/// One or more "tags" that have been associated with an objects. A tag can be any kind of Object
	/// The key difference between attachment and tag is that the former implies association by inclusion, while the latter implies associated by reference
	fn tag(&self) -> Node<Self::Object> { Node::Empty }
	/// Identifies one or more links to representations of the object
	fn url(&self) -> Node<Self::Link> { Node::Empty }
	/// Identifies an entity considered to be part of the public primary audience of an Object
	fn to(&self) -> Node<Self::Link> { Node::Empty }
	/// Identifies an Object that is part of the private primary audience of this Object
	fn bto(&self) -> Node<Self::Link> { Node::Empty }
	/// Identifies an Object that is part of the public secondary audience of this Object
	fn cc(&self) -> Node<Self::Link> { Node::Empty }
	/// Identifies one or more Objects that are part of the private secondary audience of this Object
	fn bcc(&self) -> Node<Self::Link> { Node::Empty }
	/// When used on a Link, identifies the MIME media type of the referenced resource.
	/// When used on an Object, identifies the MIME media type of the value of the content property.
	/// If not specified, the content property is assumed to contain text/html content. 
	fn media_type(&self) -> Field<&str> { Err(FieldErr("mediaType")) } // also in link
	/// When the object describes a time-bound resource, such as an audio or video, a meeting, etc, the duration property indicates the object's approximate duration.
	/// The value MUST be expressed as an xsd:duration as defined by [ xmlschema11-2], section 3.3.6 (e.g. a period of 5 seconds is represented as "PT5S"). 
	fn duration(&self) -> Field<&str> { Err(FieldErr("duration")) } // TODO how to parse xsd:duration ?

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn sensitive(&self) -> Field<bool> { Err(FieldErr("sensitive")) }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn quote_url(&self) -> Node<Self::Object> { Node::Empty }

	#[cfg(feature = "activitypub-fe")]
	fn liked_by_me(&self) -> Field<bool> { Err(FieldErr("likedByMe")) }

	#[cfg(feature = "ostatus")]
	fn conversation(&self) -> Node<Self::Object> { Node::Empty }

	fn as_activity(&self) -> Result<&Self::Activity, FieldErr> { Err(FieldErr("type")) }
	fn as_actor(&self) -> Result<&Self::Actor, FieldErr> { Err(FieldErr("type")) }
	fn as_collection(&self) -> Result<&Self::Collection, FieldErr> { Err(FieldErr("type")) }
	fn as_document(&self) -> Result<&Self::Document, FieldErr> { Err(FieldErr("type")) }
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

	crate::getter! { objectType -> type ObjectType }
	crate::getter! { attachment -> node <Self as Object>::Object }
	crate::getter! { attributedTo -> node Self::Actor }
	crate::getter! { audience -> node Self::Actor }
	crate::getter! { content -> &str }
	crate::getter! { context -> node <Self as Object>::Object }
	crate::getter! { name -> &str }
	crate::getter! { endTime -> chrono::DateTime<chrono::Utc> }
	crate::getter! { generator -> node Self::Actor }
	crate::getter! { icon -> node Self::Document }
	crate::getter! { image -> node Self::Document }
	crate::getter! { inReplyTo -> node <Self as Object>::Object }
	crate::getter! { location -> node <Self as Object>::Object }
	crate::getter! { preview -> node <Self as Object>::Object }
	crate::getter! { published -> chrono::DateTime<chrono::Utc> }
	crate::getter! { updated -> chrono::DateTime<chrono::Utc> }
	crate::getter! { replies -> node Self::Collection }
	crate::getter! { likes -> node Self::Collection }
	crate::getter! { shares -> node Self::Collection }
	crate::getter! { startTime -> chrono::DateTime<chrono::Utc> }
	crate::getter! { summary -> &str }
	crate::getter! { tag -> node <Self as Object>::Object }
	crate::getter! { to -> node Self::Link }
	crate::getter! { bto -> node Self::Link }
	crate::getter! { cc -> node Self::Link }
	crate::getter! { bcc -> node Self::Link }
	crate::getter! { mediaType -> &str }
	crate::getter! { duration -> &str }
	crate::getter! { url -> node Self::Link }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { sensitive -> bool }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { quoteUrl -> node <Self as Object>::Object }

	#[cfg(feature = "activitypub-fe")]
	crate::getter! { likedByMe -> bool }

	#[cfg(feature = "ostatus")]
	crate::getter! { conversation -> node <Self as Object>::Object }

	fn as_activity(&self) -> Result<&Self::Activity, FieldErr> {
		match self.object_type()? {
			ObjectType::Activity(_) => Ok(self),
			_ => Err(FieldErr("type")),
		}
	}

	fn as_actor(&self) -> Result<&Self::Actor, FieldErr> {
		match self.object_type()? {
			ObjectType::Actor(_) => Ok(self),
			_ => Err(FieldErr("type")),
		}
	}

	fn as_collection(&self) -> Result<&Self::Collection, FieldErr> {
		match self.object_type()? {
			ObjectType::Collection(_) => Ok(self),
			_ => Err(FieldErr("type")),
		}
	}

	fn as_document(&self) -> Result<&Self::Document, FieldErr> {
		match self.object_type()? {
			ObjectType::Document(_) => Ok(self),
			_ => Err(FieldErr("type")),
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
	crate::setter! { attributedTo -> node Self::Actor }
	crate::setter! { audience -> node Self::Actor }
	crate::setter! { content -> &str }
	crate::setter! { context -> node <Self as Object>::Object }
	crate::setter! { name -> &str }
	crate::setter! { endTime -> chrono::DateTime<chrono::Utc> }
	crate::setter! { generator -> node Self::Actor }
	crate::setter! { icon -> node Self::Document }
	crate::setter! { image -> node Self::Document }
	crate::setter! { inReplyTo -> node <Self as Object>::Object }
	crate::setter! { location -> node <Self as Object>::Object }
	crate::setter! { preview -> node <Self as Object>::Object }
	crate::setter! { published -> chrono::DateTime<chrono::Utc> }
	crate::setter! { updated -> chrono::DateTime<chrono::Utc> }
	crate::setter! { replies -> node Self::Collection }
	crate::setter! { likes -> node Self::Collection }
	crate::setter! { shares -> node Self::Collection }
	crate::setter! { startTime -> chrono::DateTime<chrono::Utc> }
	crate::setter! { summary -> &str }
	crate::setter! { tag -> node <Self as Object>::Object }
	crate::setter! { to -> node Self::Link }
	crate::setter! { bto -> node Self::Link}
	crate::setter! { cc -> node Self::Link }
	crate::setter! { bcc -> node Self::Link }
	crate::setter! { mediaType -> &str }
	crate::setter! { duration -> &str }
	crate::setter! { url -> node Self::Link }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { sensitive -> bool }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { quoteUrl -> node <Self as Object>::Object }

	#[cfg(feature = "activitypub-fe")]
	crate::setter! { likedByMe -> bool }

	#[cfg(feature = "ostatus")]
	crate::setter! { conversation -> node <Self as Object>::Object }
}
