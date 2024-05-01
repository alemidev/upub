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

	#[cfg(feature = "activitypub-fe")]
	fn following_me(&self) -> Option<bool> { None }
	#[cfg(feature = "activitypub-fe")]
	fn followed_by_me(&self) -> Option<bool> { None }

	#[cfg(feature = "activitypub-counters")]
	fn followers_count(&self) -> Option<u64> { None }
	#[cfg(feature = "activitypub-counters")]
	fn following_count(&self) -> Option<u64> { None }
	#[cfg(feature = "activitypub-counters")]
	fn statuses_count(&self) -> Option<u64> { None }

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

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn set_moved_to(self, val: Node<Self::Actor>) -> Self;
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn set_manually_approves_followers(self, val: Option<bool>) -> Self;

	#[cfg(feature = "activitypub-fe")]
	fn set_following_me(self, val: Option<bool>) -> Self;
	#[cfg(feature = "activitypub-fe")]
	fn set_followed_by_me(self, val: Option<bool>) -> Self;

	#[cfg(feature = "activitypub-counters")]
	fn set_followers_count(self, val: Option<u64>) -> Self;
	#[cfg(feature = "activitypub-counters")]
	fn set_following_count(self, val: Option<u64>) -> Self;
	#[cfg(feature = "activitypub-counters")]
	fn set_statuses_count(self, val: Option<u64>) -> Self;

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

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { moved_to::movedTo -> node Self::Actor }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { manually_approves_followers::manuallyApprovedFollowers -> bool }

	#[cfg(feature = "activitypub-fe")]
	crate::getter! { following_me::followingMe -> bool }
	#[cfg(feature = "activitypub-fe")]
	crate::getter! { followed_by_me::followedByMe -> bool }

	#[cfg(feature = "activitypub-counters")]
	crate::getter! { following_count::followingCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::getter! { followers_count::followersCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::getter! { statuses_count::statusesCount -> u64 }

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

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { moved_to::movedTo -> node Self::Actor }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { manually_approves_followers::manuallyApprovedFollowers -> bool }

	#[cfg(feature = "activitypub-fe")]
	crate::setter! { following_me::followingMe -> bool }
	#[cfg(feature = "activitypub-fe")]
	crate::setter! { followed_by_me::followedByMe -> bool }

	#[cfg(feature = "activitypub-counters")]
	crate::setter! { following_count::followingCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::setter! { followers_count::followersCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::setter! { statuses_count::statusesCount -> u64 }

	fn set_endpoints(mut self, _val: Node<<Self as Object>::Object>) -> Self {
		self.as_object_mut().unwrap().insert("endpoints".to_string(), serde_json::Value::Object(serde_json::Map::default()));
		self
	}
}
