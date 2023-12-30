use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
	#[serde(rename = "@context")]
	pub context: Vec<String>,  // note: must be @context
	pub id: String,
	#[serde(rename = "type")]
	pub actor_type: ActorType,
	#[serde(rename = "preferredUsername")]
	pub preferred_username: String,
	pub inbox: String,
	#[serde(rename = "publicKey")]
	pub public_key: PublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
	pub id: String,
	pub owner: String,
	#[serde(rename = "publicKeyPem")]
	pub public_key_pem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
	Person,
}

#[cfg(test)]
mod test {
	use super::{Actor, ActorType, PublicKey};

	#[test]
	fn actor_serializes_as_expected() {
		let actor = Actor {
			context: vec![
				"https://www.w3.org/ns/activitystreams".into(),
				"https://w3id.org/security/v1".into()
			],
			id: "https://my-example.com/actor".into(),
			actor_type: ActorType::Person,
			preferred_username: "alice".into(),
			inbox: "https://my-example.com/inbox".into(),
			public_key: PublicKey {
				id: "https://my-example.com/actor#main-key".into(),
				owner: "https://my-example.com/actor".into(),
				public_key_pem: "-----BEGIN PUBLIC KEY-----...-----END PUBLIC KEY-----".into(),
			},
		};

		let serialized_actor = serde_json::to_string(&actor).unwrap();
		let expected_serialized_actor = "{\"@context\":[\"https://www.w3.org/ns/activitystreams\",\"https://w3id.org/security/v1\"],\"id\":\"https://my-example.com/actor\",\"type\":\"Person\",\"preferredUsername\":\"alice\",\"inbox\":\"https://my-example.com/inbox\",\"publicKey\":{\"id\":\"https://my-example.com/actor#main-key\",\"owner\":\"https://my-example.com/actor\",\"publicKeyPem\":\"-----BEGIN PUBLIC KEY-----...-----END PUBLIC KEY-----\"}}";

		assert_eq!(expected_serialized_actor, serialized_actor);
	}
}
