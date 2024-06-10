pub mod accept;
pub mod ignore;
pub mod intransitive;
pub mod offer;
pub mod reject;

use crate::{Field, FieldErr, Node, Object, ObjectMut};
use accept::AcceptType;
use reject::RejectType;
use offer::OfferType;
use intransitive::IntransitiveActivityType;
use ignore::IgnoreType;

#[cfg(feature = "litepub")]
crate::strenum! {
	pub enum ActivityType {
		Activity,
		Add,
		Announce,
		Create,
		Delete,
		Dislike,
		EmojiReact,
		Flag,
		Follow,
		Join,
		Leave,
		Like,
		Listen,
		Move,
		Read,
		Remove,
		Undo,
		Update,
		View;

		IntransitiveActivity(IntransitiveActivityType),
		Accept(AcceptType),
		Ignore(IgnoreType),
		Offer(OfferType),
		Reject(RejectType)
	};
}

#[cfg(not(feature = "litepub"))]
crate::strenum! {
	pub enum ActivityType {
		Activity,
		Add,
		Announce,
		Create,
		Delete,
		Dislike,
		Flag,
		Follow,
		Join,
		Leave,
		Like,
		Listen,
		Move,
		Read,
		Remove,
		Undo,
		Update,
		View;

		IntransitiveActivity(IntransitiveActivityType),
		Accept(AcceptType),
		Ignore(IgnoreType),
		Offer(OfferType),
		Reject(RejectType)
	};
}

pub trait Activity : Object {
	fn activity_type(&self) -> Field<ActivityType> { Err(FieldErr("type")) }
	/// Describes one or more entities that either performed or are expected to perform the activity.
	/// Any single activity can have multiple actors. The actor MAY be specified using an indirect Link. 
	fn actor(&self) -> Node<Self::Actor> { Node::Empty }
	/// Describes an object of any kind.
	/// The Object type serves as the base type for most of the other kinds of objects defined in the Activity Vocabulary, including other Core types such as Activity, IntransitiveActivity, Collection and OrderedCollection. 
	fn object(&self) -> Node<Self::Object> { Node::Empty }
	/// Describes the indirect object, or target, of the activity.
	/// The precise meaning of the target is largely dependent on the type of action being described but will often be the object of the English preposition "to".
	/// For instance, in the activity "John added a movie to his wishlist", the target of the activity is John's wishlist. An activity can have more than one target. 
	fn target(&self) -> Node<Self::Object> { Node::Empty }
	/// Describes the result of the activity.
	/// For instance, if a particular action results in the creation of a new resource, the result property can be used to describe that new resource. 
	fn result(&self) -> Node<Self::Object> { Node::Empty }
	/// Describes an indirect object of the activity from which the activity is directed.
	/// The precise meaning of the origin is the object of the English preposition "from".
	/// For instance, in the activity "John moved an item to List B from List A", the origin of the activity is "List A".
	fn origin(&self) -> Node<Self::Object> { Node::Empty }
	/// Identifies one or more objects used (or to be used) in the completion of an Activity.
	fn instrument(&self) -> Node<Self::Object> { Node::Empty }
}

pub trait ActivityMut : ObjectMut {
	fn set_activity_type(self, val: Option<ActivityType>) -> Self;
	fn set_actor(self, val: Node<Self::Actor>) -> Self;
	fn set_object(self, val: Node<Self::Object>) -> Self;
	fn set_target(self, val: Node<Self::Object>) -> Self;
	fn set_result(self, val: Node<Self::Object>) -> Self;
	fn set_origin(self, val: Node<Self::Object>) -> Self;
	fn set_instrument(self, val: Node<Self::Object>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Activity for serde_json::Value {
	crate::getter! { activity_type -> type ActivityType }
	crate::getter! { actor -> node Self::Actor }
	crate::getter! { object -> node <Self as Object>::Object }
	crate::getter! { target -> node <Self as Object>::Object }
	crate::getter! { result -> node <Self as Object>::Object }
	crate::getter! { origin -> node <Self as Object>::Object }
	crate::getter! { instrument -> node <Self as Object>::Object }
}

#[cfg(feature = "unstructured")]
impl ActivityMut for serde_json::Value {
	crate::setter! { activity_type -> type ActivityType }
	crate::setter! { actor -> node Self::Actor }
	crate::setter! { object -> node <Self as Object>::Object }
	crate::setter! { target -> node <Self as Object>::Object }
	crate::setter! { result -> node <Self as Object>::Object }
	crate::setter! { origin -> node <Self as Object>::Object }
	crate::setter! { instrument -> node <Self as Object>::Object }
}
