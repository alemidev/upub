use apb::{LD, ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::{Path, Query, State}, http::HeaderMap, response::{IntoResponse, Redirect, Response}, Form};
use hmac::{Hmac, Mac};
use reqwest::Method;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use upub::{traits::Fetcher, Context};

use crate::{builders::JsonLD, ApiError, AuthIdentity, Identity};


pub async fn view(
	headers: HeaderMap,
	State(ctx): State<Context>,
) -> crate::ApiResult<Response> {
	if let Some(accept) = headers.get("Accept") {
		if let Ok(accept) = accept.to_str() {
			if accept.contains("text/html") && !accept.contains("application/ld+json") {
				return Ok(Redirect::to("/web").into_response());
			}
		}
	}
	Ok(JsonLD(
		apb::new()
			.set_id(Some(&upub::url!(ctx, "")))
			.set_actor_type(Some(apb::ActorType::Application))
			.set_name(Some(&ctx.cfg().instance.name))
			.set_summary(Some(&ctx.cfg().instance.description))
			.set_inbox(apb::Node::link(upub::url!(ctx, "/inbox")))
			.set_outbox(apb::Node::link(upub::url!(ctx, "/outbox")))
			.set_published(Some(ctx.actor().published))
			.set_endpoints(apb::Node::Empty)
			.set_preferred_username(Some(ctx.domain()))
			.set_url(apb::Node::link(upub::url!(ctx, "/")))
			.set_public_key(apb::Node::object(
				apb::new()
					.set_id(Some(&upub::url!(ctx, "#main-key")))
					.set_owner(Some(&upub::url!(ctx, "")))
					.set_public_key_pem(&ctx.actor().public_key)
			))
			.ld_context()
	).into_response())
}

pub async fn proxy_path(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(uri): Path<String>,
) -> crate::ApiResult<impl IntoResponse> {
	let query = uriproxy::expand(&uri)
		.ok_or_else(crate::ApiError::bad_request)?;
	proxy(ctx, query, auth).await
}

#[derive(Debug, serde::Deserialize)]
pub struct ProxyQuery {
	uri: String,
}

pub async fn proxy_get(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<ProxyQuery>,
) -> crate::ApiResult<impl IntoResponse> {
	proxy(ctx, query.uri, auth).await
}

pub async fn proxy_form(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Form(query): Form<String>,
) -> crate::ApiResult<impl IntoResponse> {
	proxy(ctx, query, auth).await
}

pub async fn proxy_hmac(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(hmac): Path<String>,
	Path(uri): Path<String>,
) -> crate::ApiResult<impl IntoResponse> {
	let bytes = URL_SAFE.decode(hmac).map_err(|_| ApiError::bad_request())?;
	let uri =
		std::str::from_utf8(
			&URL_SAFE.decode(uri).map_err(|_| ApiError::bad_request())?
		)
		.map_err(|_| ApiError::bad_request())?
		.to_string();

	type HmacSha256 = Hmac<sha2::Sha256>;
	let mut mac = HmacSha256::new_from_slice(ctx.cfg().security.proxy_secret.as_bytes())
		.map_err(|_| ApiError::internal_server_error())?;

	mac.update(uri.as_bytes());
	mac.verify_slice(&bytes)
		.map_err(|_| ApiError::forbidden())?;

	proxy(ctx, uri, auth).await
}

async fn proxy(ctx: Context, query: String, auth: Identity) -> crate::ApiResult<impl IntoResponse> {
	// only local users can request fetches
	if !ctx.cfg().security.allow_public_debugger && !auth.is_local() {
		return Err(crate::ApiError::unauthorized());
	}

	let resp = Context::request(
			Method::GET,
			&query,
			None,
			ctx.base(),
			ctx.pkey(),
			&format!("{}+proxy", ctx.domain()),
		)
			.await?
			.error_for_status()?;

	Ok((
		resp.headers().clone(),
		resp.bytes().await?.to_vec(),
	))
}
