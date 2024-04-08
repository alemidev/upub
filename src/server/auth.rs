use std::collections::BTreeMap;

use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts, StatusCode}};
use openssl::hash::MessageDigest;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{model, server::Context};

#[derive(Debug, Clone)]
pub enum Identity {
	Anonymous,
	Local(String),
	Remote(String),
}

pub struct AuthIdentity(pub Identity);

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthIdentity
where
	Context: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = StatusCode;

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
				Ok(None) => return Err(StatusCode::UNAUTHORIZED),
				Err(e) => {
					tracing::error!("failed querying user session: {e}");
					return Err(StatusCode::INTERNAL_SERVER_ERROR)
				},
			}
		}

		// if let Some(sig) = parts
		// 	.headers
		// 	.get("Signature")
		// 	.map(|v| v.to_str().unwrap_or(""))
		// {
		// 	let signature = HttpSignature::try_from(sig)?;
		// 	let user_id = signature.key_id.split('#').next().unwrap_or("").to_string();
		// 	let data : String = signature.headers.iter()
		// 		.map(|header| {
		// 			if header == "(request-target)" {
		// 				format!("(request-target): {} {}", parts.method, parts.uri)
		// 			} else {
		// 				format!(
		// 					"{header}: {}",
		// 					parts.headers.get(header)
		// 						.map(|h| h.to_str().unwrap_or(""))
		// 						.unwrap_or("")
		// 				)
		// 			}
		// 		})
		// 		.collect::<Vec<String>>() // TODO can we avoid this unneeded allocation?
		// 		.join("\n");

		// 	let user = ctx.fetch().user(&user_id).await.map_err(|_e| StatusCode::UNAUTHORIZED)?;
		// 	let pubkey = PKey::public_key_from_pem(user.public_key.as_bytes()).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
		// 	let mut verifier = Verifier::new(signature.digest(), &pubkey).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
		// 	verifier.update(data.as_bytes()).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
		// 	if verifier.verify(signature.signature.as_bytes()).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)? {
		// 		identity = Identity::Remote(user_id);
		// 	} else {
		// 		return Err(StatusCode::FORBIDDEN);
		// 	}
		// }

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

