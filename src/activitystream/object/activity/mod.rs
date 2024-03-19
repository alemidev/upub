pub mod accept;
pub use accept::{Accept, AcceptType};

pub mod ignore;
pub use ignore::{Ignore, IgnoreType};

pub mod intransitive;
pub use intransitive::{IntransitiveActivity, IntransitiveActivityType};

pub mod offer;
pub use offer::{Offer, OfferType};

pub mod reject;
pub use reject::{Reject, RejectType};

use crate::activitystream::node::NodeExtractor;
use crate::activitystream::Node;
use crate::strenum;

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
		View

		IntransitiveActivity(IntransitiveActivityType),
		Accept(AcceptType),
		Ignore(IgnoreType),
		Offer(OfferType),
		Reject(RejectType)
	}
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

impl Activity for serde_json::Value {
	fn activity_type(&self) -> Option<ActivityType> {
		let serde_json::Value::String(t) = self.get("type")? else { return None };
		ActivityType::try_from(t.as_str()).ok()
	}

	fn object(&self) -> Node<impl super::Object> {
		self.node_vec("object")
	}

	fn actor(&self) -> Node<impl super::Actor> {
		self.node_vec("actor")
	}
}
