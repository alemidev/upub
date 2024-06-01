use apb::{LD, ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::{Query, State}, http::HeaderMap, response::{IntoResponse, Redirect, Response}, Form, Json};
use reqwest::Method;
use upub::Context;

use crate::{builders::JsonLD, AuthIdentity};


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
pub struct FetchPath {
	id: String,
}

pub async fn proxy_get(
	State(ctx): State<Context>,
	Query(query): Query<FetchPath>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<Json<serde_json::Value>> {
	// only local users can request fetches
	if !ctx.cfg().security.allow_public_debugger && !auth.is_local() {
		return Err(crate::ApiError::unauthorized());
	}
	todo!()
	// Ok(Json(
	// 	Context::request(
	// 		Method::GET,
	// 		&query.id,
	// 		None,
	// 		ctx.base(),
	// 		ctx.pkey(),
	// 		&format!("{}+proxy", ctx.domain()),
	// 	)
	// 		.await?
	// 		.json::<serde_json::Value>()
	// 		.await?
	// ))
}

pub async fn proxy_form(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Form(query): Form<FetchPath>,
) -> crate::ApiResult<Json<serde_json::Value>> {
	// only local users can request fetches
	if !ctx.cfg().security.allow_public_debugger && auth.is_local() {
		return Err(crate::ApiError::unauthorized());
	}
	todo!()
	// Ok(Json(
	// 	Context::request(
	// 		Method::GET,
	// 		&query.id,
	// 		None,
	// 		ctx.base(),
	// 		ctx.pkey(),
	// 		&format!("{}+proxy", ctx.domain()),
	// 	)
	// 		.await?
	// 		.json::<serde_json::Value>()
	// 		.await?
	// ))
}
