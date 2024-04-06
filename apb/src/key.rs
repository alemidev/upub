// TODO technically this is not part of ActivityStreams

use crate::{getter, setter};


pub trait PublicKey : super::Base {
	fn owner(&self) -> Option<&str> { None }
	fn public_key_pem(&self) -> &str;
}

pub trait PublicKeyMut : super::BaseMut {
	fn set_owner(self, val: Option<&str>) -> Self;
	fn set_public_key_pem(self, val: &str) -> Self;
}

impl PublicKey for serde_json::Value {
	getter! { owner -> &str }

	fn public_key_pem(&self) -> &str {
		self.get("publicKeyPem").unwrap().as_str().unwrap()
	}
}

impl PublicKeyMut for serde_json::Value {
	setter! { owner -> &str }

	fn set_public_key_pem(mut self, val: &str) -> Self {
		self.as_object_mut().unwrap().insert(
			"publicKeyPem".to_string(),
			serde_json::Value::String(val.to_string()),
		);
		self
	}
}
