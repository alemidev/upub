// TODO technically this is not part of ActivityStreams

pub trait PublicKey : super::Base {
	fn owner(&self) -> Option<&str> { None }
	fn public_key_pem(&self) -> &str;
}

pub trait PublicKeyMut : super::BaseMut {
	fn set_owner(self, val: Option<&str>) -> Self;
	fn set_public_key_pem(self, val: &str) -> Self;
}

#[cfg(feature = "unstructured")]
impl PublicKey for serde_json::Value {
	crate::getter! { owner -> &str }

	fn public_key_pem(&self) -> &str {
		self.get("publicKeyPem").map(|x| x.as_str().unwrap_or_default()).unwrap_or_default()
	}
}

#[cfg(feature = "unstructured")]
impl PublicKeyMut for serde_json::Value {
	crate::setter! { owner -> &str }

	fn set_public_key_pem(mut self, val: &str) -> Self {
		self.as_object_mut().unwrap().insert(
			"publicKeyPem".to_string(),
			serde_json::Value::String(val.to_string()),
		);
		self
	}
}
