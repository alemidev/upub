pub mod accept;
pub mod ignore;
pub mod intransitive;
pub mod offer;
pub mod reject;

use crate::activitystream::Node;
use crate::strenum;
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
	fn target(&self) -> Option<&str> { None }
	fn result(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn origin(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn instrument(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
}

pub trait ActivityMut : super::ObjectMut {
	fn set_activity_type(&mut self, val: Option<ActivityType>) -> &mut Self;
	fn set_actor(&mut self, val: Node<impl super::Actor>) -> &mut Self;
	fn set_object(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_target(&mut self, val: Option<&str>) -> &mut Self;
	fn set_result(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_origin(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_instrument(&mut self, val: Node<impl super::Object>) -> &mut Self;
}

impl Activity for serde_json::Value {
	fn activity_type(&self) -> Option<ActivityType> {
		let serde_json::Value::String(t) = self.get("type")? else { return None };
		ActivityType::try_from(t.as_str()).ok()
	}
}
