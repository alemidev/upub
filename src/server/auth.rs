use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts}};
use base64::Engine;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Verifier};
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
			let http_signature = HttpSignature::parse(sig);

			let user_id = http_signature.key_id.replace("#main-key", "");
			match ctx.fetch().user(&user_id).await {
				Ok(user) => {
					let to_sign = http_signature.build_string(parts);
					// TODO assert payload's digest is equal to signature's
					match verify_control_text(&to_sign, &user.public_key, &http_signature.signature) {
						Ok(true) => identity = Identity::Remote(Context::server(&user_id)),
						Ok(false) => tracing::warn!("invalid signature"),
						Err(e) => tracing::error!("error verifying signature: {e}"),
					}
				},
				Err(e) => tracing::warn!("could not fetch user (won't verify): {e}"),
			}
		}

		Ok(AuthIdentity(identity))
	}
}



fn verify_control_text(txt: &str, key: &str, control: &str) -> crate::Result<bool> {
	let pubkey = PKey::public_key_from_pem(key.as_bytes())?;
	let mut verifier = Verifier::new(MessageDigest::sha256(), &pubkey)?;
	let signature = base64::prelude::BASE64_URL_SAFE.decode(control)?;
	Ok(verifier.verify_oneshot(&signature, txt.as_bytes())?)
}











#[derive(Debug, Clone, Default)]
pub struct HttpSignature {
	key_id: String,
	algorithm: String,
	headers: Vec<String>,
	signature: String,
}

impl HttpSignature {
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

	pub fn build_string(&self, parts: &Parts) -> String {
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
		out.join("\n")
	}

	pub fn digest(&self) -> MessageDigest {
		match self.algorithm.as_str() {
			"rsa-sha512" => MessageDigest::sha512(),
			"rsa-sha384" => MessageDigest::sha384(),
			"rsa-sha256" => MessageDigest::sha256(),
			"rsa-sha1" => MessageDigest::sha1(),
			_ => {
				tracing::error!("unknown digest algorithm, trying with rsa-sha256");
				MessageDigest::sha256()
			}
		}
	}
}
