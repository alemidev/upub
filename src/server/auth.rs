use std::collections::BTreeMap;

use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts, HeaderMap, StatusCode}};
use base64::Engine;
use http_signature_normalization::Config;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Verifier};
use reqwest::Method;
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
			let mut signature_cfg = Config::new().mastodon_compat();
			let mut headers : BTreeMap<String, String> = [
				("Signature".to_string(), sig.to_string()),
				("Host".to_string(), header_get(&parts.headers, "Host")),
				("Date".to_string(), header_get(&parts.headers, "Date")),
			].into();

			if parts.method == Method::POST {
				signature_cfg = signature_cfg.require_header("digest");
				headers.insert("Digest".to_string(), header_get(&parts.headers, "Digest"));
			}

			let unverified = match signature_cfg.begin_verify(
				parts.method.as_str(),
				parts.uri.path_and_query().map(|x| x.as_str()).unwrap_or("/"),
				headers
			) {
				Ok(x) => x,
				Err(e) => {
					tracing::error!("failed preparing signature verification context: {e}");
					return Err(UpubError::internal_server_error());
				}
			};

			let user_id = unverified.key_id().replace("#main-key", "");
			if let Ok(user) = ctx.fetch().user(&user_id).await {
				let pubkey = PKey::public_key_from_pem(user.public_key.as_bytes())?;
				
				let valid = unverified.verify(|sig, to_sign| {
					let mut verifier = Verifier::new(MessageDigest::sha256(), &pubkey).unwrap();
					verifier.update(to_sign.as_bytes())?;
					Ok(verifier.verify(&base64::prelude::BASE64_URL_SAFE.decode(sig).unwrap_or_default())?) as crate::Result<bool>
				})?;

				if !valid {
					return Err(UpubError::unauthorized());
				}

				// TODO assert payload's digest is equal to signature's

				// TODO introduce hardened mode which identifies remotes by user and not server
				identity = Identity::Remote(Context::server(&user_id));
			}
		}

		Ok(AuthIdentity(identity))
	}
}

#[allow(unused)] // TODO am i gonna reimplement http signatures for verification?
pub struct HttpSignature {
	key_id: String,
	algorithm: String,
	headers: Vec<String>,
	signature: String,
}

impl HttpSignature {
	#[allow(unused)] // TODO am i gonna reimplement http signatures for verification?
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

impl TryFrom<&str> for HttpSignature {
	type Error = StatusCode; // TODO: quite ad hoc...

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let parameters : BTreeMap<String, String> = value
			.split(',')
			.filter_map(|s| { // TODO kinda ugly, can be made nicer?
				let (k, v) = s.split_once("=\"")?;
				let (k, mut v) = (k.to_string(), v.to_string());
				v.pop();
				Some((k, v))
			}).collect();

		let sig = HttpSignature {
			key_id: parameters.get("keyId").ok_or(StatusCode::BAD_REQUEST)?.to_string(),
			algorithm: parameters.get("algorithm").ok_or(StatusCode::BAD_REQUEST)?.to_string(),
			headers: parameters.get("headers").map(|x| x.split(' ').map(|x| x.to_string()).collect()).unwrap_or(vec!["date".to_string()]),
			signature: parameters.get("signature").ok_or(StatusCode::BAD_REQUEST)?.to_string(),
		};

		Ok(sig)
	}
}


pub fn header_get(headers: &HeaderMap, k: &str) -> String {
	headers.get(k).map(|x| x.to_str().unwrap_or("")).unwrap_or("").to_string()
}
