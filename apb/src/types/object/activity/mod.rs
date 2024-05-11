pub mod accept;
pub mod ignore;
pub mod intransitive;
pub mod offer;
pub mod reject;

use crate::{Node, Object, ObjectMut};
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
	fn activity_type(&self) -> Option<ActivityType> { None }
	fn actor(&self) -> Node<Self::Actor> { Node::Empty }
	fn object(&self) -> Node<Self::Object> { Node::Empty }
	fn target(&self) -> Node<Self::Object> { Node::Empty }
	fn result(&self) -> Node<Self::Object> { Node::Empty }
	fn origin(&self) -> Node<Self::Object> { Node::Empty }
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
