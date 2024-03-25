use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts, StatusCode}};
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

		if let Some(sig) = parts
			.headers
			.get("Signature")
			.map(|v| v.to_str().unwrap_or(""))
		{
			// TODO load pub key of actor and decode+verify signature
			let decoded = "asd".to_string();

			let mut key_id = None;
			let mut headers = None;
			let mut signature = None;
			for frag in decoded.split(',') {
				if frag.starts_with("keyId=") {
					key_id = Some(frag.replace("keyId=\"", ""));
					key_id.as_mut().unwrap().pop();
				}
				if frag.starts_with("signature=") {
					signature = Some(frag.replace("signature=\"", ""));
					signature.as_mut().unwrap().pop();
				}
				if frag.starts_with("headers=") {
					let mut h = frag.replace("headers=\"", "");
					h.pop();
					headers = Some(h.split(' ').map(|x| x.to_string()).collect::<Vec<String>>());
				}
			}

			if key_id.is_none() || headers.is_none() || signature.is_none() {
				tracing::warn!("malformed signature");
				return Err(StatusCode::BAD_REQUEST);
			}
		}

		Ok(AuthIdentity(identity))
	}
}
