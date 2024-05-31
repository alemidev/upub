use std::collections::BTreeMap;

use axum::http::request::Parts;
use base64::Engine;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Verifier};

#[derive(Debug, Clone, Default)]
pub struct HttpSignature {
	pub key_id: String,
	pub algorithm: String,
	pub headers: Vec<String>,
	pub signature: String,
	pub control: String,
}

impl HttpSignature {
	pub fn new(key_id: String, algorithm: String, headers: &[&str]) -> Self {
		HttpSignature {
			key_id, algorithm,
			headers: headers.iter().map(|x| x.to_string()).collect(),
			signature: String::new(),
			control: String::new(),
		}
	}

	pub fn parse(header: &str) -> Self {
		let mut sig = HttpSignature::default();
		header.split(',')
			.filter_map(|x| x.split_once('='))
			.map(|(k, v)| (k, v.trim_end_matches('"').trim_matches('"')))
			.for_each(|(k, v)| match k {
				"keyId" => sig.key_id = v.to_string(),
				"algorithm" => sig.algorithm = v.to_string(),
				"signature" => sig.signature = v.to_string(),
				"headers" => sig.headers = v.split(' ').map(|x| x.to_string()).collect(),
				_ => tracing::warn!("unexpected field in http signature: '{k}=\"{v}\"'"),
			});
		sig
	}

	pub fn header(&self) -> String {
		format!(
			"keyId=\"{}\",algorithm=\"{}\",headers=\"{}\",signature=\"{}\"",
			self.key_id, self.algorithm, self.headers.join(" "), self.signature,
		)
	}

	pub fn build_manually(&mut self, method: &str, target: &str, mut headers: BTreeMap<String, String>) -> &mut Self {
		let mut out = Vec::new();
		for header in &self.headers {
			match header.as_str() {
				"(request-target)" => out.push(format!("(request-target): {method} {target}")),
				// TODO other pseudo-headers
				_ => out.push(
					format!("{header}: {}", headers.remove(header).unwrap_or_default())
				),
			}
		}
		self.control = out.join("\n");
		self
	}

	pub fn build_from_parts(&mut self, parts: &Parts) -> &mut Self {
		let mut out = Vec::new();
		for header in self.headers.iter() {
			match header.as_str() {
				"(request-target)" => out.push(
					format!(
						"(request-target): {} {}",
						parts.method.to_string().to_lowercase(),
						parts.uri.path_and_query().map(|x| x.as_str()).unwrap_or("/")
					)
				),
				// TODO other pseudo-headers,
				_ => out.push(format!("{}: {}",
					header.to_lowercase(),
					parts.headers.get(header).map(|x| x.to_str().unwrap_or("")).unwrap_or("")
				)),
			}
		}
		self.control = out.join("\n");
		self
	}

	pub fn verify(&self, key: &str) -> crate::Result<bool> {
		let pubkey = PKey::public_key_from_pem(key.as_bytes())?;
		let mut verifier = Verifier::new(MessageDigest::sha256(), &pubkey)?;
		let signature = base64::prelude::BASE64_STANDARD.decode(&self.signature)?;
		Ok(verifier.verify_oneshot(&signature, self.control.as_bytes())?)
	}

	pub fn sign(&mut self, key: &str) -> crate::Result<&str> {
		let privkey = PKey::private_key_from_pem(key.as_bytes())?;
		let mut signer = openssl::sign::Signer::new(MessageDigest::sha256(), &privkey)?;
		signer.update(self.control.as_bytes())?;
		self.signature = base64::prelude::BASE64_STANDARD.encode(signer.sign_to_vec()?);
		Ok(&self.signature)
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn http_signature_signs_and_verifies() {
		let key = openssl::rsa::Rsa::generate(2048).unwrap();
		let private_key = std::str::from_utf8(&key.private_key_to_pem().unwrap()).unwrap().to_string();
		let public_key = std::str::from_utf8(&key.public_key_to_pem().unwrap()).unwrap().to_string();
		let mut signer = super::HttpSignature {
			key_id: "test".to_string(),
			algorithm: "rsa-sha256".to_string(),
			headers: vec![
				"(request-target)".to_string(),
				"host".to_string(),
				"date".to_string(),
			],
			signature: String::new(),
			control: String::new(),
		};

		signer
			.build_manually("get", "/actor/inbox", [("host".into(), "example.net".into()), ("date".into(), "Sat, 13 Apr 2024 13:36:23 GMT".into())].into())
			.sign(&private_key)
			.unwrap();

		let mut verifier = super::HttpSignature::parse(&signer.header());
		verifier.build_manually("get", "/actor/inbox", [("host".into(), "example.net".into()), ("date".into(), "Sat, 13 Apr 2024 13:36:23 GMT".into())].into());

		assert!(verifier.verify(&public_key).unwrap());
	}
}
