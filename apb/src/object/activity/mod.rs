pub mod accept;
pub mod ignore;
pub mod intransitive;
pub mod offer;
pub mod reject;

use crate::{Node, object::{Object, ObjectMut}, getter, setter, strenum};
use accept::AcceptType;
use reject::RejectType;
use offer::OfferType;
use intransitive::IntransitiveActivityType;
use ignore::IgnoreType;

strenum! {
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

impl Activity for serde_json::Value {
	getter! { activity_type -> type ActivityType }
	getter! { actor -> node Self::Actor }
	getter! { object -> node <Self as Object>::Object }
	getter! { target -> node <Self as Object>::Object }
	getter! { result -> node <Self as Object>::Object }
	getter! { origin -> node <Self as Object>::Object }
	getter! { instrument -> node <Self as Object>::Object }
}

impl ActivityMut for serde_json::Value {
	setter! { activity_type -> type ActivityType }
	setter! { actor -> node Self::Actor }
	setter! { object -> node <Self as Object>::Object }
	setter! { target -> node <Self as Object>::Object }
	setter! { result -> node <Self as Object>::Object }
	setter! { origin -> node <Self as Object>::Object }
	setter! { instrument -> node <Self as Object>::Object }
}
