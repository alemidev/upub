use apb::{LD, ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::{Path, Query, State}, http::HeaderMap, response::{IntoResponse, Redirect, Response}};
use reqwest::Method;
use sea_orm::{Condition, ColumnTrait};
use upub::{traits::{Cloaker, Fetcher}, Context};

use crate::{builders::JsonLD, ApiError, AuthIdentity, Identity};

use super::{PaginatedSearch, Pagination};


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

pub async fn search(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<PaginatedSearch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	if !auth.is_local() && ctx.cfg().security.allow_public_search {
		return Err(crate::ApiError::forbidden());
	}

	let mut filter = Condition::any()
		.add(auth.filter());

	if let Identity::Local { ref id, .. } = auth {
		filter = filter.add(upub::model::object::Column::AttributedTo.eq(id));
	}

	filter = Condition::all()
		.add(upub::model::object::Column::Content.like(page.q))
		.add(filter);

	// TODO lmao rethink this all
	let page = Pagination {
		offset: page.offset,
		batch: page.batch,
	};

	crate::builders::paginate_feed(
		upub::url!(ctx, "/search"),
		filter,
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
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
