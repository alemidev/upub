use axum::{extract::{FromRef, FromRequestParts}, http::{header, request::Parts}};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use httpsign::HttpSignature;
use upub::traits::{fetch::PullError, Fetcher};

use crate::ApiError;

#[derive(Debug, Clone)]
pub enum Identity {
	Anonymous,
	Remote {
		user: String,
		domain: String,
		internal: i64,
	},
	Local {
		id: String,
		internal: i64,
	},
}

impl Identity {
	pub fn filter_activities(&self) -> Condition {
		let base_cond = Condition::any().add(upub::model::addressing::Column::Actor.is_null());
		match self {
			Identity::Anonymous => base_cond,
			Identity::Remote { internal, .. } => base_cond.add(upub::model::addressing::Column::Instance.eq(*internal)), 
			Identity::Local { internal, .. } => base_cond.add(upub::model::addressing::Column::Actor.eq(*internal)),
		}
	}

	pub fn filter_objects(&self) -> Condition {
		let base_cond = Condition::any().add(upub::model::addressing::Column::Actor.is_null());
		match self {
			Identity::Anonymous => base_cond,
			Identity::Remote { internal, .. } => base_cond.add(upub::model::addressing::Column::Instance.eq(*internal)),
			// TODO should we allow all users on same server to see? or just specific user??
			Identity::Local { id, internal } => base_cond
				.add(upub::model::addressing::Column::Actor.eq(*internal))
				.add(upub::model::object::Column::AttributedTo.eq(id)),
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
	upub::Context: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let ctx = upub::Context::from_ref(state);
		let mut identity = Identity::Anonymous;

		let auth_header = parts
			.headers
			.get(header::AUTHORIZATION)
			.map(|v| v.to_str().unwrap_or(""))
			.unwrap_or("");

		if auth_header.starts_with("Bearer ") {
			match upub::model::session::Entity::find()
				.filter(upub::model::session::Column::Secret.eq(auth_header.replace("Bearer ", "")))
				.filter(upub::model::session::Column::Expires.gt(chrono::Utc::now()))
				.one(ctx.db())
				.await?
			{
				None => return Err(ApiError::unauthorized()),
				Some(x) => {
					// TODO could we store both actor ap id and internal id in session? to avoid this extra
					// lookup on *every* local authed request we receive...
					let internal = upub::model::actor::Entity::ap_to_internal(&x.actor, ctx.db())
						.await?
						.ok_or_else(ApiError::internal_server_error)?;
					identity = Identity::Local { id: x.actor, internal };
				},
			}
		}

		if let Some(sig) = parts
			.headers
			.get("Signature")
			.map(|v| v.to_str().unwrap_or(""))
		{
			tracing::debug!("validating http signature '{sig}'");
			let mut http_signature = HttpSignature::parse(sig);

			// TODO assert payload's digest is equal to signature's
			//      really annoying to do here because we're streaming
			//      the request, maybe even impossible with this design?

			let user_id = http_signature.key_id
				.replace("/main-key", "") // gotosocial whyyyyy
				.split('#')
				.next().ok_or(ApiError::bad_request())?
				.to_string();

			match ctx.fetch_user(&user_id, ctx.db()).await {
				Err(PullError::Database(x)) => return Err(PullError::Database(x).into()),
				Err(e) => tracing::debug!("could not fetch {user_id} to verify signature: {e}"),
				Ok(user) => {
					let signature = http_signature.build_from_parts(parts);
					tracing::debug!("constructed http signature {signature:?}");
					let valid = signature.verify(&user.public_key)?;

					if !valid {
						tracing::warn!("refusing mismatching http signature");
						return Err(ApiError::unauthorized());
					}

					let internal = upub::model::instance::Entity::domain_to_internal(&user.domain, ctx.db())
						.await?
						.ok_or_else(ApiError::internal_server_error)?; // user but not their domain???
					identity = Identity::Remote { user: user.id, domain: user.domain, internal };
				},
			}

		}

		Ok(AuthIdentity(identity))
	}
}
