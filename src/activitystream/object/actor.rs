use crate::{activitystream::{Base, BaseType}, strenum};

use super::ObjectType;

strenum! {
	pub enum ActorType {
		Application,
		Group,
		Organization,
		Person,
		Object
	}
}

pub trait Profile : super::Object {
	// not a Node because it's always embedded and one
	fn describes(&self) -> Option<impl super::Object> { None::<serde_json::Value> }
}

pub trait Actor : super::Object {
	fn actor_type(&self) -> Option<ActorType> { None }
}

impl Actor for serde_json::Value {
	fn actor_type(&self) -> Option<ActorType> {
		match self.base_type()? {
			BaseType::Object(ObjectType::Actor(x)) => Some(x),
			_ => None,
		}
	}
}

impl Profile for serde_json::Value {

}
