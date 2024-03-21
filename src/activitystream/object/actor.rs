use crate::{activitystream::Node, getter, setter, strenum};

use super::collection::Collection;
use super::super::key::PublicKey;

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
	fn preferred_username(&self) -> Option<&str> { None }
	fn inbox(&self) -> Node<impl Collection>;
	fn outbox(&self) -> Node<impl Collection>;
	fn following(&self) -> Node<impl Collection> { Node::empty() }
	fn followers(&self) -> Node<impl Collection> { Node::empty() }
	fn liked(&self) -> Node<impl Collection> { Node::empty() }
	fn streams(&self) -> Node<impl Collection> { Node::empty() }
	fn endpoints(&self) -> Option<serde_json::Map<String, String>> { None }
	fn public_key(&self) -> Node<impl PublicKey> { Node::empty() }
}

pub trait ActorMut : super::ObjectMut {
	fn set_actor_type(self, val: Option<ActorType>) -> Self;
	fn set_preferred_username(self, val: Option<&str>) -> Self;
	fn set_inbox(self, val: Node<impl Collection>) -> Self;
	fn set_outbox(self, val: Node<impl Collection>) -> Self;
	fn set_following(self, val: Node<impl Collection>) -> Self;
	fn set_followers(self, val: Node<impl Collection>) -> Self;
	fn set_liked(self, val: Node<impl Collection>) -> Self;
	fn set_streams(self, val: Node<impl Collection>) -> Self;
	fn set_endpoints(self, val: Option<serde_json::Map<String, String>>) -> Self;
	fn set_public_key(self, val: Node<impl PublicKey>) -> Self;
}

impl Actor for serde_json::Value {
	getter! { actor_type -> type ActorType }
	getter! { preferred_username::preferredUsername -> &str }
	getter! { inbox -> node impl Collection }
	getter! { outbox -> node impl Collection }
	getter! { following -> node impl Collection }
	getter! { followers -> node impl Collection }
	getter! { liked -> node impl Collection }
	getter! { streams -> node impl Collection }
	getter! { public_key::publicKey -> node impl PublicKey }

	fn endpoints(&self) -> Option<serde_json::Map<String, String>> {
		todo!()
	}
}

impl ActorMut for serde_json::Value {
	setter! { actor_type -> type ActorType }
	setter! { preferred_username::preferredUsername -> &str }
	setter! { inbox -> node impl Collection }
	setter! { outbox -> node impl Collection }
	setter! { following -> node impl Collection }
	setter! { followers -> node impl Collection }
	setter! { liked -> node impl Collection }
	setter! { streams -> node impl Collection }
	setter! { public_key::publicKey -> node impl PublicKey }

	fn set_endpoints(self, _val: Option<serde_json::Map<String, String>>) -> Self {
		todo!()
	}
}
