use std::collections::BTreeMap;

use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts}};
use base64::Engine;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Verifier};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{errors::UpubError, model, server::Context};

#[derive(Debug, Clone)]
pub enum Identity {
	Anonymous,
	Local(String),
	Remote(String),
}

impl Identity {
	pub fn filter_condition(&self) -> Condition {
		let base_cond = Condition::any().add(model::addressing::Column::Actor.eq(apb::target::PUBLIC));
		match self {
			Identity::Anonymous => base_cond,
			Identity::Local(uid) => base_cond.add(model::addressing::Column::Actor.eq(uid)),
			Identity::Remote(server) => base_cond.add(model::addressing::Column::Server.eq(server)),
			// TODO should we allow all users on same server to see? or just specific user??
		}
	}

	pub fn is_anon(&self) -> bool {
		match self {
			Self::Anonymous => true,
			_ => false,
		}
	}

	pub fn is_user(&self, uid: &str) -> bool {
		match self {
			Self::Local(x) => x == uid,
			_ => false,
		}
	}

	pub fn is_server(&self, uid: &str) -> bool {
		match self {
			Self::Remote(x) => x == uid,
			_ => false,
		}
	}
}

pub struct AuthIdentity(pub Identity);

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthIdentity
where
	Context: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = UpubError;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let ctx = Context::from_ref(state);
		let mut identity = Identity::Anonymous;

		let auth_header = parts
			.headers
			.get(header::AUTHORIZATION)
			.map(|v| v.to_str().unwrap_or(""))
			.unwrap_or("");

		if auth_header.starts_with("Bearer ") {
			match model::session::Entity::find_by_id(auth_header.replace("Bearer ", ""))
				.filter(Condition::all().add(model::session::Column::Expires.gt(chrono::Utc::now())))
				.one(ctx.db())
				.await
			{
				Ok(Some(x)) => identity = Identity::Local(x.actor),
				Ok(None) => return Err(UpubError::unauthorized()),
				Err(e) => {
					tracing::error!("failed querying user session: {e}");
					return Err(UpubError::internal_server_error())
				},
			}
		}

		if let Some(sig) = parts
			.headers
			.get("Signature")
			.map(|v| v.to_str().unwrap_or(""))
		{
			let mut http_signature = HttpSignature::parse(sig);

			// TODO assert payload's digest is equal to signature's
			let user_id = http_signature.key_id
				.split('#')
				.next().ok_or(UpubError::bad_request())?
				.to_string();

			match ctx.fetch().user(&user_id).await {
				Ok(user) => match http_signature
						.build_from_parts(parts)
						.verify(&user.public_key)
					{
						Ok(true) => identity = Identity::Remote(Context::server(&user_id)),
						Ok(false) => tracing::warn!("invalid signature"),
						Err(e) => tracing::error!("error verifying signature: {e}"),
					},
				Err(e) => {
					// since most activities are deletions for users we never saw, let's handle this case
					// if while fetching we receive a GONE, it means we didn't have this user and it doesn't
					// exist anymore, so it must be a deletion we can ignore
					if let UpubError::Reqwest(ref x) = e {
						if let Some(StatusCode::GONE) = x.status() {
							return Err(UpubError::not_modified());
						}
					}
					tracing::warn!("could not fetch user (won't verify): {e}");
				}
			}
		}

		Ok(AuthIdentity(identity))
	}
}


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
