use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts}};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{errors::UpubError, model, server::Context};

use super::{fetcher::Fetcher, httpsign::HttpSignature};

#[derive(Debug, Clone)]
pub enum Identity {
	Anonymous,
	Remote {
		domain: String,
		internal: i64,
	},
	Local {
		id: String,
		internal: i64,
	},
}

impl Identity {
	pub fn filter_condition(&self) -> Condition {
		let base_cond = Condition::any().add(model::addressing::Column::Actor.eq(apb::target::PUBLIC));
		match self {
			Identity::Anonymous => base_cond,
			Identity::Remote { internal, .. } => base_cond.add(model::addressing::Column::Instance.eq(*internal)),
			// TODO should we allow all users on same server to see? or just specific user??
			Identity::Local { id, internal } => base_cond
				.add(model::addressing::Column::Actor.eq(*internal))
				.add(model::activity::Column::Actor.eq(id))
				.add(model::object::Column::AttributedTo.eq(id)),
		}
	}

	pub fn my_id(&self) -> Option<i64> {
		match self {
			Identity::Local { internal, .. } => Some(*internal),
			_ => None,
		}
	}

	pub fn is(&self, uid: &str) -> bool {
		match self {
			Identity::Anonymous => false,
			Identity::Remote { .. } => false, // TODO per-actor server auth should check this
			Identity::Local { id, .. } => id.as_str() == uid
		}
	}

	#[allow(unused)]
	pub fn is_anon(&self) -> bool {
		matches!(self, Self::Anonymous)
	}

	#[allow(unused)]
	pub fn is_local(&self) -> bool {
		matches!(self, Self::Local { .. })
	}

	#[allow(unused)]
	pub fn is_remote(&self) -> bool {
		matches!(self, Self::Remote { .. })
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
			match model::session::Entity::find()
				.filter(model::session::Column::Secret.eq(auth_header.replace("Bearer ", "")))
				.filter(model::session::Column::Expires.gt(chrono::Utc::now()))
				.one(ctx.db())
				.await
			{
				Ok(None) => return Err(UpubError::unauthorized()),
				Ok(Some(x)) => {
					// TODO could we store both actor ap id and internal id in session? to avoid this extra
					// lookup on *every* local authed request we receive...
					let internal = model::actor::Entity::ap_to_internal(&x.actor, ctx.db()).await?;
					identity = Identity::Local { id: x.actor, internal };
				},
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
						Ok(true) => {
							// TODO can we avoid this extra db rountrip made on each server fetch?
							let domain = Context::server(&user_id); 
							// TODO this will fail because we never fetch and insert into instance oops
							let internal = model::instance::Entity::domain_to_internal(&domain, ctx.db()).await?;
							identity = Identity::Remote { domain, internal };
						},
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
