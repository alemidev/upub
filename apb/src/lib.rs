//! # apb
//! > traits and types for implementing [ActivityPub](https://www.w3.org/TR/activitypub/)
//!
//! The main type this crate exposes is the [Node], which can be:
//!  - [Node::Empty]: not present in object
//!  - [Node::Link]: contains just link to object
//!  - [Node::Object]: contains embedded object
//!  - [Node::Array]: contains array of embedded objects
//!
//! Nodes contain AP objects, which implement one or more traits (such as [Object] or [Actor])
//!
//! ## features
//! * `unstructured`: all traits are implemented for [serde_json::Value], so that it's possible to manipulate free-form json maps as valid AP objects
//! * `orm`: enum types are also database-friendly with sea-orm
//! * `fetch`: [Node] exposes [Node::fetch] to dereference remote nodes
//!
//! ## structure
//! - **[Base]** | **[BaseMut]** | [BaseType]
//!   - [BaseType::Link] | **[Link]** | **[LinkMut]** | [LinkType]
//!     - [LinkType::Mention]
//!     - [LinkType::Link]
//!   - [BaseType::Object] | **[Object]** | **[ObjectMut]** | [ObjectType]
//!     - [ObjectType::Activity] | **[Activity]** | **[ActivityMut]** | [ActivityType]
//!       - [ActivityType::Accept] | **[Accept]** | **[AcceptMut]** | [AcceptType]
//!         - [AcceptType::TentativeAccept]
//!       - [ActivityType::Add]
//!       - [ActivityType::Announce]
//!       - [ActivityType::Create]
//!       - [ActivityType::Delete]
//!       - [ActivityType::Dislike]
//!       - [ActivityType::Flag]
//!       - [ActivityType::Follow]
//!       - [ActivityType::IntransitiveActivity] | **[IntransitiveActivity]** | **[IntransitiveActivityMut]** | [IntransitiveActivityType]
//!         - [IntransitiveActivityType::IntransitiveActivity]
//!         - [IntransitiveActivityType::Arrive]
//!         - [IntransitiveActivityType::Question]
//!         - [IntransitiveActivityType::Travel]
//!       - [ActivityType::Ignore] | **[Ignore]** | **[IgnoreMut]** | [IgnoreType]
//!         - [IgnoreType::Ignore]
//!         - [IgnoreType::Block]
//!       - [ActivityType::Join]
//!       - [ActivityType::Leave]
//!       - [ActivityType::Like]
//!       - [ActivityType::Listen]
//!       - [ActivityType::Move]
//!       - [ActivityType::Offer] | **[Offer]** | **[OfferMut]** | [OfferType]
//!           - [OfferType::Offer]
//!           - [OfferType::Invite]
//!       - [ActivityType::Read]
//!       - [ActivityType::Reject] | **[Reject]** | **[RejectMut]** | [RejectType]
//!           - [RejectType::Reject]
//!           - [RejectType::TentativeReject]
//!       - [ActivityType::Remove]
//!       - [ActivityType::Undo]
//!       - [ActivityType::Update]
//!       - [ActivityType::View]
//!     - [ObjectType::Actor] | **[Actor]** | **[ActorMut]** | [ActorType] *
//!       - [ActorType::Application]
//!       - [ActorType::Group]
//!       - [ActorType::Organization]
//!       - [ActorType::Person]
//!     - [ObjectType::Article]
//!     - [ObjectType::Collection] | **[Collection]** | **[CollectionMut]** | [CollectionType]
//!       - [CollectionType::Collection]
//!       - [CollectionType::CollectionPage]
//!       - [CollectionType::OrderedCollection]
//!       - [CollectionType::OrderedCollectionPage]
//!     - [ObjectType::Document] | **[Document]** | **[DocumentMut]** | [DocumentType]
//!       - [DocumentType::Document]
//!       - [DocumentType::Audio]
//!       - [DocumentType::Image]
//!       - [DocumentType::Page]
//!       - [DocumentType::Video]
//!     - [ObjectType::Event]
//!     - [ObjectType::Note]
//!     - [ObjectType::Object]
//!     - [ObjectType::Place]
//!     - [ObjectType::Profile]
//!     - [ObjectType::Relationship]
//!     - [ObjectType::Tombstone]
//!   - **[PublicKey]** | **[PublicKeyMut]** \*\*
//!
//! *: `Actor` is technically just an object, not really a "subtype"
//!
//! **: `PublicKey` is introduced in ActivityPub, it's not part of ActivityStream
//!



mod macros;
pub(crate) use macros::strenum;

#[cfg(feature = "unstructured")]
pub(crate) use macros::{getter, setter};

mod node;
pub use node::Node;

pub mod target;

mod key;
pub use key::{PublicKey, PublicKeyMut};

pub mod field;
pub use field::{Field, FieldErr};

#[cfg(feature = "shortcuts")]
pub mod shortcuts;
#[cfg(feature = "shortcuts")]
pub use shortcuts::Shortcuts;

#[cfg(feature = "jsonld")]
pub mod jsonld;

#[cfg(feature = "jsonld")]
pub use jsonld::LD;

mod types;
pub use types::{
	base::{Base, BaseMut, BaseType},
	link::{Link, LinkMut, LinkType},
	object::{
		Object, ObjectMut, ObjectType,
		activity::{
			Activity, ActivityMut, ActivityType,
			accept::{Accept, AcceptMut, AcceptType},
			ignore::{Ignore, IgnoreMut, IgnoreType},
			intransitive::{IntransitiveActivity, IntransitiveActivityMut, IntransitiveActivityType},
			offer::{Offer, OfferMut, OfferType},
			reject::{Reject, RejectMut, RejectType},
		},
		actor::{Actor, ActorMut, ActorType, Endpoints, EndpointsMut},
		collection::{
			Collection, CollectionMut, CollectionType,
			page::{CollectionPage, CollectionPageMut}
		},
		document::{Document, DocumentMut, DocumentType},
		place::{Place, PlaceMut},
		profile::Profile,
		relationship::{Relationship, RelationshipMut},
		tombstone::{Tombstone, TombstoneMut},
	},
};

#[cfg(feature = "unstructured")]
pub fn new() -> serde_json::Value {
	serde_json::Value::Object(serde_json::Map::default())
}

#[cfg(feature = "fetch")]
pub use reqwest;
