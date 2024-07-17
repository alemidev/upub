use apb::{LD, ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::{Path, Query, State}, http::HeaderMap, response::{IntoResponse, Redirect, Response}};
use reqwest::Method;
use upub::{traits::{Cloaker, Fetcher}, Context};

use crate::{builders::JsonLD, ApiError, ApiResult, AuthIdentity, Identity};


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

#[derive(Debug, serde::Deserialize)]
pub struct ProxyQuery {
	uri: String,
}

pub async fn ap_fetch(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<ProxyQuery>,
) -> crate::ApiResult<axum::Json<serde_json::Value>> {
	// only local users can request fetches
	if !ctx.cfg().security.allow_public_debugger && !auth.is_local() {
		return Err(crate::ApiError::unauthorized());
	}

	let resp = Context::request(
			Method::GET,
			&query.uri,
			None,
			ctx.base(),
			ctx.pkey(),
			&format!("{}+fetch", ctx.domain()),
		)
			.await?
			.error_for_status()?;
	
	
	Ok(axum::Json(resp.json().await?))
}

pub async fn cloak_proxy(
	State(ctx): State<Context>,
	Path((hmac, uri)): Path<(String, String)>,
) -> crate::ApiResult<impl IntoResponse> {
	let uri = ctx.uncloak(&hmac, &uri)
		.ok_or_else(ApiError::unauthorized)?;

	let resp = Context::request(
			Method::GET,
			&uri,
			None,
			ctx.base(),
			ctx.pkey(),
			&format!("{}+proxy", ctx.domain()),
		)
			.await?
			.error_for_status()?;
	
	let headers = resp.headers().clone();
	let body = resp.bytes().await?.to_vec();

	if serde_json::from_slice::<serde_json::Value>(&body).is_ok() {
		return Err(ApiError::forbidden());
	}

	Ok((headers, body))
}
