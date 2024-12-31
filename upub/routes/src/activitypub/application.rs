use apb::{LD, ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::{Path, Query, State}, http::HeaderMap, response::{IntoResponse, Redirect, Response}};
use reqwest::Method;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use upub::{selector::{RichFillable, RichObject}, traits::{Cloaker, Fetcher}, Context};

use crate::{builders::JsonLD, ApiError, AuthIdentity};

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
			.set_id(Some(upub::url!(ctx, "")))
			.set_actor_type(Some(apb::ActorType::Application))
			.set_name(Some(ctx.cfg().instance.name.clone()))
			.set_summary(Some(ctx.cfg().instance.description.clone()))
			.set_inbox(apb::Node::link(upub::url!(ctx, "/inbox")))
			.set_outbox(apb::Node::link(upub::url!(ctx, "/outbox")))
			.set_published(Some(ctx.actor().published))
			.set_endpoints(apb::Node::Empty)
			.set_preferred_username(Some(ctx.domain().to_string()))
			.set_url(apb::Node::link(upub::url!(ctx, "/")))
			.set_public_key(apb::Node::object(
				apb::new()
					.set_id(Some(upub::url!(ctx, "#main-key")))
					.set_owner(Some(upub::url!(ctx, "")))
					.set_public_key_pem(ctx.actor().public_key.clone())
			))
			.ld_context()
	).into_response())
}

pub async fn search(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<PaginatedSearch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	if !auth.is_local() && !ctx.cfg().security.allow_public_search {
		return Err(crate::ApiError::forbidden());
	}

	let filter = Condition::all()
		.add(auth.filter_activities())
		.add(upub::model::object::Column::Content.like(format!("%{}%", page.q)));

	// TODO lmao rethink this all
	//      still haven't redone this gg me
	//      have redone it but didnt rethink it properly so we're stuck with this bahahaha
	let page = Pagination {
		offset: page.offset,
		batch: page.batch,
		replies: Some(true),
	};

	let (limit, offset) = page.pagination();
	let items = upub::Query::feed(auth.my_id(), true)
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.order_by_desc(upub::model::addressing::Column::Published)
		.order_by_desc(upub::model::activity::Column::Internal)
		.into_model::<RichObject>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/search"), page, apb::Node::array(items))
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
	let _user; // need this for lifetimes

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

	if upub::ext::is_blacklisted(&query.uri, &ctx.cfg().reject.fetch) {
		return Err(crate::ApiError::FetchError(upub::traits::fetch::RequestError::AbortedForPolicy));
	}

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

pub async fn cloak_proxy(
	State(ctx): State<Context>,
	Path((hmac, uri)): Path<(String, String)>,
) -> crate::ApiResult<impl IntoResponse> {
	let uri = ctx.uncloak(&hmac, &uri)
		.ok_or_else(ApiError::unauthorized)?;

	if upub::ext::is_blacklisted(&uri, &ctx.cfg().reject.media) {
		return Err(ApiError::Status(axum::http::StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS));
	}

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
