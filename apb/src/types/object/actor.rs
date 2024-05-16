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
	type Endpoints : Endpoints;

	fn actor_type(&self) -> Option<ActorType> { None }
	fn preferred_username(&self) -> Option<&str> { None }
	fn inbox(&self) -> Node<Self::Collection>;
	fn outbox(&self) -> Node<Self::Collection>;
	fn following(&self) -> Node<Self::Collection> { Node::Empty }
	fn followers(&self) -> Node<Self::Collection> { Node::Empty }
	fn liked(&self) -> Node<Self::Collection> { Node::Empty }
	fn streams(&self) -> Node<Self::Collection> { Node::Empty }
	fn endpoints(&self) -> Node<Self::Endpoints> { Node::Empty }
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

pub trait Endpoints : Object {
	/// Endpoint URI so this actor's clients may access remote ActivityStreams objects which require authentication to access. To use this endpoint, the client posts an x-www-form-urlencoded id parameter with the value being the id of the requested ActivityStreams object. 
	fn proxy_url(&self) -> Option<&str> { None }
	/// If OAuth 2.0 bearer tokens [RFC6749] [RFC6750] are being used for authenticating client to server interactions, this endpoint specifies a URI at which a browser-authenticated user may obtain a new authorization grant. 
	fn oauth_authorization_endpoint(&self) -> Option<&str> { None }
	/// If OAuth 2.0 bearer tokens [RFC6749] [RFC6750] are being used for authenticating client to server interactions, this endpoint specifies a URI at which a client may acquire an access token. 
	fn oauth_token_endpoint(&self) -> Option<&str> { None }
	/// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which browser-authenticated users may authorize a client's public key for client to server interactions. 
	fn provide_client_key(&self) -> Option<&str> { None }
	/// If Linked Data Signatures and HTTP Signatures are being used for authentication and authorization, this endpoint specifies a URI at which a client key may be signed by the actor's key for a time window to act on behalf of the actor in interacting with foreign servers. 
	fn sign_client_key(&self) -> Option<&str> { None }
	/// An optional endpoint used for wide delivery of publicly addressed activities and activities sent to followers. sharedInbox endpoints SHOULD also be publicly readable OrderedCollection objects containing objects addressed to the Public special collection. Reading from the sharedInbox endpoint MUST NOT present objects which are not addressed to the Public endpoint.
	fn shared_inbox(&self) -> Option<&str> { None }
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

	crate::getter! { actor_type -> type ActorType }
	crate::getter! { preferred_username::preferredUsername -> &str }
	crate::getter! { inbox -> node Self::Collection }
	crate::getter! { outbox -> node Self::Collection }
	crate::getter! { following -> node Self::Collection }
	crate::getter! { followers -> node Self::Collection }
	crate::getter! { liked -> node Self::Collection }
	crate::getter! { streams -> node Self::Collection }
	crate::getter! { public_key::publicKey -> node Self::PublicKey }
	crate::getter! { endpoints -> node Self::Endpoints }

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
}

#[cfg(feature = "unstructured")]
impl Endpoints for serde_json::Value {
	crate::getter! { proxy_url::proxyUrl -> &str }
	crate::getter! { oauth_authorization_endpoint::oauthAuthorizationEndpoint -> &str }
	crate::getter! { oauth_token_endpoint::oauthTokenEndpoint -> &str }
	crate::getter! { provide_client_key::provideClientKey -> &str }
	crate::getter! { sign_client_key::signClientKey -> &str }
	crate::getter! { shared_inbox::sharedInbox -> &str }
}

#[cfg(feature = "unstructured")]
impl ActorMut for serde_json::Value {
	type PublicKey = serde_json::Value;
	type Endpoints = serde_json::Value;

	crate::setter! { actor_type -> type ActorType }
	crate::setter! { preferred_username::preferredUsername -> &str }
	crate::setter! { inbox -> node Self::Collection }
	crate::setter! { outbox -> node Self::Collection }
	crate::setter! { following -> node Self::Collection }
	crate::setter! { followers -> node Self::Collection }
	crate::setter! { liked -> node Self::Collection }
	crate::setter! { streams -> node Self::Collection }
	crate::setter! { public_key::publicKey -> node Self::PublicKey }
	crate::setter! { endpoints -> node Self::Endpoints }
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
}

#[cfg(feature = "unstructured")]
impl EndpointsMut for serde_json::Value {
	crate::setter! { proxy_url::proxyUrl -> &str }
	crate::setter! { oauth_authorization_endpoint::oauthAuthorizationEndpoint -> &str }
	crate::setter! { oauth_token_endpoint::oauthTokenEndpoint -> &str }
	crate::setter! { provide_client_key::provideClientKey -> &str }
	crate::setter! { sign_client_key::signClientKey -> &str }
	crate::setter! { shared_inbox::sharedInbox -> &str }
}
