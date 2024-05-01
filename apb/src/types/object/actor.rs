use crate::{Node, Object, ObjectMut};

crate::strenum! {
	pub enum ActorType {
		Application,
		Group,
		Organization,
		Person,
		Service;
	};
}

pub trait Actor : Object {
	type PublicKey : crate::PublicKey;

	fn actor_type(&self) -> Option<ActorType> { None }
	fn preferred_username(&self) -> Option<&str> { None }
	fn inbox(&self) -> Node<Self::Collection>;
	fn outbox(&self) -> Node<Self::Collection>;
	fn following(&self) -> Node<Self::Collection> { Node::Empty }
	fn followers(&self) -> Node<Self::Collection> { Node::Empty }
	fn liked(&self) -> Node<Self::Collection> { Node::Empty }
	fn streams(&self) -> Node<Self::Collection> { Node::Empty }
	fn endpoints(&self) -> Node<Self::Object> { Node::Empty }
	fn public_key(&self) -> Node<Self::PublicKey> { Node::Empty }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn moved_to(&self) -> Node<Self::Actor> { Node::Empty }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn manually_approves_followers(&self) -> Option<bool> { None }

	// idk about this? everyone has it but AP doesn't mention it
	fn discoverable(&self) -> Option<bool> { None }
}

pub trait ActorMut : ObjectMut {
	type PublicKey : crate::PublicKey;

	fn set_actor_type(self, val: Option<ActorType>) -> Self;
	fn set_preferred_username(self, val: Option<&str>) -> Self;
	fn set_inbox(self, val: Node<Self::Collection>) -> Self;
	fn set_outbox(self, val: Node<Self::Collection>) -> Self;
	fn set_following(self, val: Node<Self::Collection>) -> Self;
	fn set_followers(self, val: Node<Self::Collection>) -> Self;
	fn set_liked(self, val: Node<Self::Collection>) -> Self;
	fn set_streams(self, val: Node<Self::Collection>) -> Self;
	fn set_endpoints(self, val: Node<Self::Object>) -> Self; // TODO it's more complex than this!
	fn set_public_key(self, val: Node<Self::PublicKey>) -> Self;
	fn set_discoverable(self, val: Option<bool>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Actor for serde_json::Value {
	type PublicKey = serde_json::Value;

	crate::getter! { actor_type -> type ActorType }
	crate::getter! { preferred_username::preferredUsername -> &str }
	crate::getter! { inbox -> node Self::Collection }
	crate::getter! { outbox -> node Self::Collection }
	crate::getter! { following -> node Self::Collection }
	crate::getter! { followers -> node Self::Collection }
	crate::getter! { liked -> node Self::Collection }
	crate::getter! { streams -> node Self::Collection }
	crate::getter! { public_key::publicKey -> node Self::PublicKey }
	crate::getter! { discoverable -> bool }

	fn endpoints(&self) -> Node<<Self as Object>::Object> {
		todo!()
	}
}

#[cfg(feature = "unstructured")]
impl ActorMut for serde_json::Value {
	type PublicKey = serde_json::Value;

	crate::setter! { actor_type -> type ActorType }
	crate::setter! { preferred_username::preferredUsername -> &str }
	crate::setter! { inbox -> node Self::Collection }
	crate::setter! { outbox -> node Self::Collection }
	crate::setter! { following -> node Self::Collection }
	crate::setter! { followers -> node Self::Collection }
	crate::setter! { liked -> node Self::Collection }
	crate::setter! { streams -> node Self::Collection }
	crate::setter! { public_key::publicKey -> node Self::PublicKey }
	crate::setter! { discoverable -> bool }

	fn set_endpoints(mut self, _val: Node<<Self as Object>::Object>) -> Self {
		self.as_object_mut().unwrap().insert("endpoints".to_string(), serde_json::Value::Object(serde_json::Map::default()));
		self
	}
}
