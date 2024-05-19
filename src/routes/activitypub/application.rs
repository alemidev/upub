use apb::{ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::{Query, State}, http::HeaderMap, response::{IntoResponse, Redirect, Response}, Json};
use reqwest::Method;

use crate::{errors::UpubError, server::{auth::{AuthIdentity, Identity}, fetcher::Fetcher, Context}, url};

use super::{jsonld::LD, JsonLD};


pub async fn view(
	headers: HeaderMap,
	State(ctx): State<Context>,
) -> crate::Result<Response> {
	if let Some(accept) = headers.get("Accept") {
		if let Ok(accept) = accept.to_str() {
			if accept.contains("text/html") {
				return Ok(Redirect::to("/web").into_response());
			}
		}
	}
	Ok(JsonLD(
		serde_json::Value::new_object()
			.set_id(Some(&url!(ctx, "")))
			.set_actor_type(Some(apb::ActorType::Application))
			.set_name(Some(&ctx.cfg().instance.name))
			.set_summary(Some(&ctx.cfg().instance.description))
			.set_inbox(apb::Node::link(url!(ctx, "/inbox")))
			.set_outbox(apb::Node::link(url!(ctx, "/outbox")))
			.set_published(Some(ctx.app().created))
			.set_endpoints(apb::Node::Empty)
			.set_preferred_username(Some(ctx.domain()))
			.set_public_key(apb::Node::object(
				serde_json::Value::new_object()
					.set_id(Some(&url!(ctx, "#main-key")))
					.set_owner(Some(&url!(ctx, "")))
					.set_public_key_pem(&ctx.app().public_key)
			))
			.ld_context()
	).into_response())
}

#[derive(Debug, serde::Deserialize)]
pub struct FetchPath {
	id: String,
}

pub async fn debug(
	State(ctx): State<Context>,
	Query(query): Query<FetchPath>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<Json<serde_json::Value>> {
	// only local users can request debug fetches
	if !ctx.cfg().security.allow_public_debugger && !matches!(auth, Identity::Local(_)) {
		return Err(UpubError::unauthorized());
	}
	Ok(Json(
		Context::request(
			Method::GET,
			&query.id,
			None,
			&ctx.base(),
			&ctx.app().private_key,
			ctx.domain(),
		)
			.await?
			.json::<serde_json::Value>()
			.await?
	))
}
