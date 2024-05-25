use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts}};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{errors::UpubError, model, server::Context};

use super::{fetcher::Fetcher, httpsign::HttpSignature};

#[derive(Debug, Clone)]
pub enum Identity {
	Anonymous,
	Local(i64),
	Remote(i64),
}

impl Identity {
	pub fn filter_condition(&self) -> Condition {
		let base_cond = Condition::any().add(model::addressing::Column::Actor.eq(apb::target::PUBLIC));
		match self {
			Identity::Anonymous => base_cond,
			Identity::Remote(server_id) => base_cond.add(model::addressing::Column::Instance.eq(*server_id)),
			// TODO should we allow all users on same server to see? or just specific user??
			Identity::Local(user_id) => base_cond
				.add(model::addressing::Column::Actor.eq(*user_id))
				.add(model::activity::Column::Actor.eq(*user_id))
				.add(model::object::Column::AttributedTo.eq(*user_id)),
		}
	}

	pub fn user_id(&self) -> Option<i64> {
		match self {
			Identity::Local(x) => Some(*x),
			_ => None,
		}
	}

	pub fn server_id(&self) -> Option<i64> {
		match self {
			Identity::Remote(x) => Some(*x),
			_ => None,
		}
	}

	pub fn is(&self, id: i64) -> bool {
		match self {
			Identity::Anonymous => false,
			Identity::Remote(_) => false, // TODO per-actor server auth should check this
			Identity::Local(user_id) => *user_id == id
		}
	}

	pub fn is_anon(&self) -> bool {
		matches!(self, Self::Anonymous)
	}

	pub fn is_local(&self) -> bool {
		matches!(self, Self::Local(_))
	}

	pub fn is_remote(&self) -> bool {
		matches!(self, Self::Remote(_))
	}

	pub fn is_user(&self, usr: i64) -> bool {
		self.user_id().map(|id| id == usr).unwrap_or(false)
	}

	pub fn is_server(&self, server: i64) -> bool {
		self.server_id().map(|id| id == server).unwrap_or(false)
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
			match model::session::Entity::find_by_secret(&auth_header.replace("Bearer ", ""))
				.filter(model::session::Column::Expires.gt(chrono::Utc::now()))
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

			match ctx.fetch_user(&user_id).await {
				Ok(user) => match http_signature
						.build_from_parts(parts)
						.verify(&user.public_key)
					{
						Ok(true) => identity = Identity::Remote(Context::server(&user_id)),
						Ok(false) => tracing::warn!("invalid signature: {http_signature:?}"),
						Err(e) => tracing::error!("error verifying signature: {e}"),
					},
				Err(e) => {
					// since most activities are deletions for users we never saw, let's handle this case
					// if while fetching we receive a GONE, it means we didn't have this user and it doesn't
					// exist anymore, so it must be a deletion we can ignore
					if let UpubError::Reqwest(ref x) = e {
						if let Some(StatusCode::GONE) = x.status() {
							return Err(UpubError::Status(StatusCode::OK)); // 200 so mastodon will shut uppp
						}
					}
					tracing::warn!("could not fetch user (won't verify): {e}");
				}
			}
		}

		Ok(AuthIdentity(identity))
	}
}
