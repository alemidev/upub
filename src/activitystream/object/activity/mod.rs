pub mod accept;
pub mod ignore;
pub mod intransitive;
pub mod offer;
pub mod reject;

use crate::activitystream::Node;
use crate::{getter, setter, strenum};
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

pub trait Activity : super::Object {
	fn activity_type(&self) -> Option<ActivityType> { None }
	fn actor(&self) -> Node<impl super::Actor> { Node::Empty::<serde_json::Value> }
	fn object(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn target(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn result(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn origin(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn instrument(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
}

pub trait ActivityMut : super::ObjectMut {
	fn set_activity_type(&mut self, val: Option<ActivityType>) -> &mut Self;
	fn set_actor(&mut self, val: Node<impl super::Actor>) -> &mut Self;
	fn set_object(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_target(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_result(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_origin(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_instrument(&mut self, val: Node<impl super::Object>) -> &mut Self;
}

impl Activity for serde_json::Value {
	getter! { activity_type -> type ActivityType }
	getter! { actor -> node impl super::Actor }
	getter! { object -> node impl super::Object }
	getter! { target -> node impl super::Object }
	getter! { result -> node impl super::Object }
	getter! { origin -> node impl super::Object }
	getter! { instrument -> node impl super::Object }
}

impl ActivityMut for serde_json::Value {
	setter! { activity_type -> type ActivityType }
	setter! { actor -> node impl super::Actor }
	setter! { object -> node impl super::Object }
	setter! { target -> node impl super::Object }
	setter! { result -> node impl super::Object }
	setter! { origin -> node impl super::Object }
	setter! { instrument -> node impl super::Object }
}
