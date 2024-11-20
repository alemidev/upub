// TODO technically this is not part of ActivityStreams

pub trait PublicKey : super::Base {
	fn owner(&self) -> crate::Field<String> { Err(crate::FieldErr("owner")) }
	fn public_key_pem(&self) -> String;
}

pub trait PublicKeyMut : super::BaseMut {
	fn set_owner(self, val: Option<String>) -> Self;
	fn set_public_key_pem(self, val: String) -> Self;
}

#[cfg(feature = "unstructured")]
impl PublicKey for serde_json::Value {
	crate::getter! { owner -> String }

	fn public_key_pem(&self) -> String {
		self.get("publicKeyPem").and_then(|x| x.as_str()).unwrap_or_default().to_string()
	}
}

#[cfg(feature = "unstructured")]
impl PublicKeyMut for serde_json::Value {
	crate::setter! { owner -> String }

	fn set_public_key_pem(mut self, val: String) -> Self {
		self.as_object_mut().unwrap().insert(
			"publicKeyPem".to_string(),
			serde_json::Value::String(val),
		);
		self
	}
}
