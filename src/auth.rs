use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts, StatusCode}};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{model, server::Context};

#[derive(Debug, Clone)]
pub enum Identity {
	Anonymous,
	User(String),
	Server(String),
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
				Ok(Some(x)) => identity = Identity::User(x.actor),
				Ok(None) => return Err(StatusCode::UNAUTHORIZED),
				Err(e) => {
					tracing::error!("failed querying user session: {e}");
					return Err(StatusCode::INTERNAL_SERVER_ERROR)
				},
			}
		}

		// TODO check and validate HTTP signature
	
		Ok(AuthIdentity(identity))
	}
}
