use axum::{extract::{Path, Query, State}, response::IntoResponse};
use reqwest::Method;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, SelectColumns};
use upub::{traits::{Cloaker, Fetcher}, Context};

use crate::{ApiError, AuthIdentity};


#[derive(Debug, serde::Deserialize)]
pub struct ProxyQuery {
	uri: String,
}

pub async fn activitypub(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<ProxyQuery>,
) -> crate::ApiResult<axum::Json<serde_json::Value>> {
	let _user; // need this for lifetimes

	if upub::ext::BlacklistKind::Fetch.hit(&query.uri, &ctx.cfg().reject) {
		return Err(crate::ApiError::FetchError(upub::traits::fetch::RequestError::AbortedForPolicy));
	}

	let (from, key) = match auth {
		crate::Identity::Anonymous => {
			if !ctx.cfg().security.allow_public_debugger {
				return Err(crate::ApiError::unauthorized());
			}
			(ctx.base(), ctx.pkey())
		},
		crate::Identity::Remote { .. } => return Err(crate::ApiError::forbidden()),
		crate::Identity::Local { internal, .. } => {
			_user = upub::model::actor::Entity::find_by_id(internal)
				.one(ctx.db())
				.await?;
			match _user {
				None => (ctx.base(), ctx.pkey()),
				Some(ref u) => match u.private_key {
					None => (ctx.base(), ctx.pkey()),
					Some(ref k) => (u.id.as_str(), k.as_str()),
				}
			}
		},
	};

	let resp = Context::request(
			Method::GET,
			&query.uri,
			None,
			from,
			key,
			&format!("{}+fetch", ctx.domain()),
		)
			.await?
			.error_for_status()?;
	
	
	Ok(axum::Json(resp.json().await?))
}

pub async fn cloak(
	State(ctx): State<Context>,
	Path((hmac, uri)): Path<(String, String)>,
) -> crate::ApiResult<impl IntoResponse> {
	let uri = ctx.uncloak(&hmac, &uri)
		.ok_or_else(ApiError::unauthorized)?;

	if upub::ext::BlacklistKind::Media.hit(&uri, &ctx.cfg().reject) {
		return Err(ApiError::Status(axum::http::StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS));
	}

	proxy_request(ctx, uri).await
}

pub async fn emoji(
	State(ctx): State<Context>,
	Path((domain, name)) : Path<(String, String)>,
) -> crate::ApiResult<impl IntoResponse> {
	let uri = upub::model::emoji::Entity::find()
		.filter(upub::model::emoji::Column::Name.eq(name))
		.filter(upub::model::emoji::Column::Domain.eq(domain))
		.select_only()
		.select_column(upub::model::emoji::Column::Uri)
		.into_tuple::<String>()
		.one(ctx.db())
		.await?
		.ok_or(crate::ApiError::not_found())?;

	if upub::ext::BlacklistKind::Media.hit(&uri, &ctx.cfg().reject) {
		return Err(ApiError::Status(axum::http::StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS));
	}

	proxy_request(ctx, uri).await
}

async fn proxy_request(ctx: upub::Context, uri: String) -> crate::ApiResult<impl IntoResponse> {
	let resp = Context::client(ctx.domain())
		.get(uri)
		.send()
		.await?
		.error_for_status()?;

	let headers = resp.headers().clone();
	// TODO can we stream the response body as it comes?
	let body = resp.bytes().await?.to_vec();

	// TODO not so great to just try parsing json, but this should be a cheap check as most things we
	// proxy are not json (as in, dont start with '{')
	if serde_json::from_slice::<serde_json::Value>(&body).is_ok() {
		return Err(ApiError::forbidden());
	}

	Ok((headers, body))
}
