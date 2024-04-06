use crate::{Node, getter, setter, strenum};

use super::{Object, ObjectMut, super::key::PublicKey};

strenum! {
	pub enum ActorType {
		Application,
		Group,
		Organization,
		Person;
	};
}

pub trait Actor : Object {
	type PublicKey : PublicKey;

	fn actor_type(&self) -> Option<ActorType> { None }
	fn preferred_username(&self) -> Option<&str> { None }
	fn inbox(&self) -> Node<Self::Collection>;
	fn outbox(&self) -> Node<Self::Collection>;
	fn following(&self) -> Node<Self::Collection> { todo!() }
	fn followers(&self) -> Node<Self::Collection> { todo!() }
	fn liked(&self) -> Node<Self::Collection> { todo!() }
	fn streams(&self) -> Node<Self::Collection> { todo!() }
	fn endpoints(&self) -> Option<serde_json::Map<String, String>> { None }
	fn public_key(&self) -> Node<Self::PublicKey> { todo!() }
	// idk about this? everyone has it but AP doesn't mention it
	fn discoverable(&self) -> Option<bool> { None }
}

pub trait ActorMut : ObjectMut {
	type PublicKey : PublicKey;

	fn set_actor_type(self, val: Option<ActorType>) -> Self;
	fn set_preferred_username(self, val: Option<&str>) -> Self;
	fn set_inbox(self, val: Node<Self::Collection>) -> Self;
	fn set_outbox(self, val: Node<Self::Collection>) -> Self;
	fn set_following(self, val: Node<Self::Collection>) -> Self;
	fn set_followers(self, val: Node<Self::Collection>) -> Self;
	fn set_liked(self, val: Node<Self::Collection>) -> Self;
	fn set_streams(self, val: Node<Self::Collection>) -> Self;
	fn set_endpoints(self, val: Option<serde_json::Map<String, String>>) -> Self;
	fn set_public_key(self, val: Node<Self::PublicKey>) -> Self;
	fn set_discoverable(self, val: Option<bool>) -> Self;
}

impl Actor for serde_json::Value {
	type PublicKey = serde_json::Value;

	getter! { actor_type -> type ActorType }
	getter! { preferred_username::preferredUsername -> &str }
	getter! { inbox -> node Self::Collection }
	getter! { outbox -> node Self::Collection }
	getter! { following -> node Self::Collection }
	getter! { followers -> node Self::Collection }
	getter! { liked -> node Self::Collection }
	getter! { streams -> node Self::Collection }
	getter! { public_key::publicKey -> node Self::PublicKey }
	getter! { discoverable -> bool }

	fn endpoints(&self) -> Option<serde_json::Map<String, String>> {
		todo!()
	}
}

impl ActorMut for serde_json::Value {
	type PublicKey = serde_json::Value;

	setter! { actor_type -> type ActorType }
	setter! { preferred_username::preferredUsername -> &str }
	setter! { inbox -> node Self::Collection }
	setter! { outbox -> node Self::Collection }
	setter! { following -> node Self::Collection }
	setter! { followers -> node Self::Collection }
	setter! { liked -> node Self::Collection }
	setter! { streams -> node Self::Collection }
	setter! { public_key::publicKey -> node Self::PublicKey }
	setter! { discoverable -> bool }

	fn set_endpoints(mut self, _val: Option<serde_json::Map<String, String>>) -> Self {
		self.as_object_mut().unwrap().insert("endpoints".to_string(), serde_json::Value::Object(serde_json::Map::default()));
		self
	}
}
