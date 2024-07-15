use crate::{Field, FieldErr, Node, Object, ObjectMut};

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
	type Endpoints : Endpoints;

	fn actor_type(&self) -> Field<ActorType> { Err(FieldErr("type")) }
	/// A short username which may be used to refer to the actor, with no uniqueness guarantees.
	fn preferred_username(&self) -> Field<&str> { Err(FieldErr("preferredUsername")) }
	/// A reference to an [ActivityStreams] OrderedCollection comprised of all the messages received by the actor; see 5.2 Inbox. 
	fn inbox(&self) -> Node<Self::Collection>;
	/// An [ActivityStreams] OrderedCollection comprised of all the messages produced by the actor; see 5.1 Outbox. 
	fn outbox(&self) -> Node<Self::Collection>;
	/// A link to an [ActivityStreams] collection of the actors that this actor is following; see 5.4 Following Collection
	fn following(&self) -> Node<Self::Collection> { Node::Empty }
	/// A link to an [ActivityStreams] collection of the actors that follow this actor; see 5.3 Followers Collection. 
	fn followers(&self) -> Node<Self::Collection> { Node::Empty }
	/// A link to an [ActivityStreams] collection of objects this actor has liked; see 5.5 Liked Collection.
	fn liked(&self) -> Node<Self::Collection> { Node::Empty }
	/// A list of supplementary Collections which may be of interest.
	fn streams(&self) -> Node<Self::Collection> { Node::Empty }
	/// A json object which maps additional (typically server/domain-wide) endpoints which may be useful either for this actor or someone referencing this actor.
	/// This mapping may be nested inside the actor document as the value or may be a link to a JSON-LD document with these properties. 
	fn endpoints(&self) -> Node<Self::Endpoints> { Node::Empty }
	fn public_key(&self) -> Node<Self::PublicKey> { Node::Empty } // TODO hmmm where is this from??

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn moved_to(&self) -> Node<Self::Actor> { Node::Empty }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn manually_approves_followers(&self) -> Field<bool> { Err(FieldErr("manuallyApprovesFollowers")) }

	#[cfg(feature = "did-core")]
	fn also_known_as(&self) -> Node<Self::Actor> { Node::Empty }

	#[cfg(feature = "activitypub-fe")]
	fn following_me(&self) -> Field<bool> { Err(FieldErr("followingMe")) }
	#[cfg(feature = "activitypub-fe")]
	fn followed_by_me(&self) -> Field<bool> { Err(FieldErr("followedByMe")) }
	#[cfg(feature = "activitypub-fe")]
	fn notifications(&self) -> Node<Self::Collection> { Node::Empty }

	#[cfg(feature = "activitypub-counters")]
	fn followers_count(&self) -> Field<u64> { Err(FieldErr("followersCount")) }
	#[cfg(feature = "activitypub-counters")]
	fn following_count(&self) -> Field<u64> { Err(FieldErr("followingCount")) }
	#[cfg(feature = "activitypub-counters")]
	fn statuses_count(&self) -> Field<u64> { Err(FieldErr("statusesCount")) }

	#[cfg(feature = "toot")]
	fn discoverable(&self) -> Field<bool> { Err(FieldErr("discoverable")) }
	#[cfg(feature = "toot")]
	fn featured(&self) -> Node<Self::Collection> { Node::Empty }
}

pub trait Endpoints : Object {
	/// Endpoint URI so this actor's clients may access remote ActivityStreams objects which require authentication to access. To use this endpoint, the client posts an x-www-form-urlencoded id parameter with the value being the id of the requested ActivityStreams object. 
	fn proxy_url(&self) -> Field<&str> { Err(FieldErr("proxyUrl")) }
	/// If OAuth 2.0 bearer tokens [RFC6749] [RFC6750] are being used for authenticating client to server interactions, this endpoint specifies a URI at which a browser-authenticated user may obtain a new authorization grant. 
	fn oauth_authorization_endpoint(&self) -> Field<&str> { Err(FieldErr("oauthAuthorizationEndpoint")) }
	/// If OAuth 2.0 bearer tokens [RFC6749] [RFC6750] are being used for authenticating client to server interactions, this endpoint specifies a URI at which a client may acquire an access token. 
	fn oauth_token_endpoint(&self) -> Field<&str> { Err(FieldErr("oauthTokenEndpoint")) }
	/// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which browser-authenticated users may authorize a client's public key for client to server interactions. 
	fn provide_client_key(&self) -> Field<&str> { Err(FieldErr("provideClientKey")) }
	/// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which a client key may be signed by the actor's key for a time window to act on behalf of the actor in interacting with foreign servers. 
	fn sign_client_key(&self) -> Field<&str> { Err(FieldErr("signClientKey")) }
	/// An optional endpoint used for wide delivery of publicly addressed activities and activities sent to followers. sharedInbox endpoints SHOULD also be publicly readable OrderedCollection objects containing objects addressed to the Public special collection. Reading from the sharedInbox endpoint MUST NOT present objects which are not addressed to the Public endpoint.
	fn shared_inbox(&self) -> Field<&str> { Err(FieldErr("sharedInbox")) }
}

pub trait ActorMut : ObjectMut {
	type PublicKey : crate::PublicKey;
	type Endpoints : Endpoints;

	fn set_actor_type(self, val: Option<ActorType>) -> Self;
	fn set_preferred_username(self, val: Option<&str>) -> Self;
	fn set_inbox(self, val: Node<Self::Collection>) -> Self;
	fn set_outbox(self, val: Node<Self::Collection>) -> Self;
	fn set_following(self, val: Node<Self::Collection>) -> Self;
	fn set_followers(self, val: Node<Self::Collection>) -> Self;
	fn set_liked(self, val: Node<Self::Collection>) -> Self;
	fn set_streams(self, val: Node<Self::Collection>) -> Self;
	fn set_endpoints(self, val: Node<Self::Endpoints>) -> Self;
	fn set_public_key(self, val: Node<Self::PublicKey>) -> Self;

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn set_moved_to(self, val: Node<Self::Actor>) -> Self;
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	fn set_manually_approves_followers(self, val: Option<bool>) -> Self;

	#[cfg(feature = "did-core")]
	fn set_also_known_as(self, val: Node<Self::Actor>) -> Self;

	#[cfg(feature = "activitypub-fe")]
	fn set_following_me(self, val: Option<bool>) -> Self;
	#[cfg(feature = "activitypub-fe")]
	fn set_followed_by_me(self, val: Option<bool>) -> Self;
	#[cfg(feature = "activitypub-fe")]
	fn set_notifications(self, val: Node<Self::Collection>) -> Self;

	#[cfg(feature = "activitypub-counters")]
	fn set_followers_count(self, val: Option<u64>) -> Self;
	#[cfg(feature = "activitypub-counters")]
	fn set_following_count(self, val: Option<u64>) -> Self;
	#[cfg(feature = "activitypub-counters")]
	fn set_statuses_count(self, val: Option<u64>) -> Self;

	#[cfg(feature = "toot")]
	fn set_discoverable(self, val: Option<bool>) -> Self;
	#[cfg(feature = "toot")]
	fn set_featured(self, val: Node<Self::Collection>) -> Self;
}

pub trait EndpointsMut : ObjectMut {
	/// Endpoint URI so this actor's clients may access remote ActivityStreams objects which require authentication to access. To use this endpoint, the client posts an x-www-form-urlencoded id parameter with the value being the id of the requested ActivityStreams object. 
	fn set_proxy_url(self, val: Option<&str>) -> Self;
	/// If OAuth 2.0 bearer tokens [RFC6749] [RFC6750] are being used for authenticating client to server interactions, this endpoint specifies a URI at which a browser-authenticated user may obtain a new authorization grant. 
	fn set_oauth_authorization_endpoint(self, val: Option<&str>) -> Self;
	/// If OAuth 2.0 bearer tokens [RFC6749] [RFC6750] are being used for authenticating client to server interactions, this endpoint specifies a URI at which a client may acquire an access token. 
	fn set_oauth_token_endpoint(self, val: Option<&str>) -> Self;
	/// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which browser-authenticated users may authorize a client's public key for client to server interactions. 
	fn set_provide_client_key(self, val: Option<&str>) -> Self;
	/// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which a client key may be signed by the actor's key for a time window to act on behalf of the actor in interacting with foreign servers. 
	fn set_sign_client_key(self, val: Option<&str>) -> Self;
	/// An optional endpoint used for wide delivery of publicly addressed activities and activities sent to followers. sharedInbox endpoints SHOULD also be publicly readable OrderedCollection objects containing objects addressed to the Public special collection. Reading from the sharedInbox endpoint MUST NOT present objects which are not addressed to the Public endpoint.
	fn set_shared_inbox(self, val: Option<&str>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Actor for serde_json::Value {
	type PublicKey = serde_json::Value;
	type Endpoints = serde_json::Value;

	crate::getter! { actorType -> type ActorType }
	crate::getter! { preferredUsername -> &str }
	crate::getter! { inbox -> node Self::Collection }
	crate::getter! { outbox -> node Self::Collection }
	crate::getter! { following -> node Self::Collection }
	crate::getter! { followers -> node Self::Collection }
	crate::getter! { liked -> node Self::Collection }
	crate::getter! { streams -> node Self::Collection }
	crate::getter! { publicKey -> node Self::PublicKey }
	crate::getter! { endpoints -> node Self::Endpoints }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { movedTo -> node Self::Actor }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::getter! { manuallyApprovesFollowers -> bool }

	#[cfg(feature = "did-core")]
	crate::getter! { alsoKnownAs -> node Self::Actor }

	#[cfg(feature = "activitypub-fe")]
	crate::getter! { followingMe -> bool }
	#[cfg(feature = "activitypub-fe")]
	crate::getter! { followedByMe -> bool }
	#[cfg(feature = "activitypub-fe")]
	crate::getter! { notifications -> node Self::Collection }

	#[cfg(feature = "activitypub-counters")]
	crate::getter! { followingCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::getter! { followersCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::getter! { statusesCount -> u64 }

	#[cfg(feature = "toot")]
	crate::getter! { discoverable -> bool }
	#[cfg(feature = "toot")]
	crate::getter! { featured -> node Self::Collection }
}

#[cfg(feature = "unstructured")]
impl Endpoints for serde_json::Value {
	crate::getter! { proxyUrl -> &str }
	crate::getter! { oauthAuthorizationEndpoint -> &str }
	crate::getter! { oauthTokenEndpoint -> &str }
	crate::getter! { provideClientKey -> &str }
	crate::getter! { signClientKey -> &str }
	crate::getter! { sharedInbox -> &str }
}

#[cfg(feature = "unstructured")]
impl ActorMut for serde_json::Value {
	type PublicKey = serde_json::Value;
	type Endpoints = serde_json::Value;

	crate::setter! { actor_type -> type ActorType }
	crate::setter! { preferredUsername -> &str }
	crate::setter! { inbox -> node Self::Collection }
	crate::setter! { outbox -> node Self::Collection }
	crate::setter! { following -> node Self::Collection }
	crate::setter! { followers -> node Self::Collection }
	crate::setter! { liked -> node Self::Collection }
	crate::setter! { streams -> node Self::Collection }
	crate::setter! { publicKey -> node Self::PublicKey }
	crate::setter! { endpoints -> node Self::Endpoints }

	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { movedTo -> node Self::Actor }
	#[cfg(feature = "activitypub-miscellaneous-terms")]
	crate::setter! { manuallyApprovesFollowers -> bool }


	#[cfg(feature = "did-core")]
	crate::setter! { alsoKnownAs -> node Self::Actor }

	#[cfg(feature = "activitypub-fe")]
	crate::setter! { followingMe -> bool }
	#[cfg(feature = "activitypub-fe")]
	crate::setter! { followedByMe -> bool }
	#[cfg(feature = "activitypub-fe")]
	crate::setter! { notifications -> node Self::Collection }

	#[cfg(feature = "activitypub-counters")]
	crate::setter! { followingCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::setter! { followersCount -> u64 }
	#[cfg(feature = "activitypub-counters")]
	crate::setter! { statusesCount -> u64 }

	#[cfg(feature = "toot")]
	crate::setter! { discoverable -> bool }
	#[cfg(feature = "toot")]
	crate::setter! { featured -> node Self::Collection }
}

#[cfg(feature = "unstructured")]
impl EndpointsMut for serde_json::Value {
	crate::setter! { proxyUrl -> &str }
	crate::setter! { oauthAuthorizationEndpoint -> &str }
	crate::setter! { oauthTokenEndpoint -> &str }
	crate::setter! { provideClientKey -> &str }
	crate::setter! { signClientKey -> &str }
	crate::setter! { sharedInbox -> &str }
}
