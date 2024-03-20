use crate::{activitystream::{Base, BaseType}, strenum};

use super::ObjectType;

strenum! {
	pub enum ActorType {
		Application,
		Group,
		Organization,
		Person,
		Object;
	};
}

pub trait Actor : super::Object {
	fn actor_type(&self) -> Option<ActorType> { None }
}

pub trait ActorMut : super::ObjectMut {
	fn set_actor_type(&mut self, val: Option<ActorType>) -> &mut Self;
}

impl Actor for serde_json::Value {
	fn actor_type(&self) -> Option<ActorType> {
		match self.base_type()? {
			BaseType::Object(ObjectType::Actor(x)) => Some(x),
			_ => None,
		}
	}
}
