use axum::{http::StatusCode, extract::{Path, Query, State}, Json};
use sea_orm::{ColumnTrait, Condition, QueryFilter, QuerySelect};

use upub::{selector::{RichActivity, RichFillable}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	match auth {
		Identity::Anonymous => Err(crate::ApiError::forbidden()),
		Identity::Remote { .. } => Err(crate::ApiError::forbidden()),
		Identity::Local { id: user, .. } => if ctx.uid(&id) == user {
			crate::builders::collection(upub::url!(ctx, "/actors/{id}/inbox"), None)
		} else {
			Err(crate::ApiError::forbidden())
		},
	}
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let Identity::Local { id: uid, internal } = &auth else {
		// local inbox is only for local users
		return Err(crate::ApiError::forbidden());
	};
	if uid != &ctx.uid(&id) {
		return Err(crate::ApiError::forbidden());
	}

	let filter = Condition::any()
		.add(upub::model::addressing::Column::Actor.eq(*internal))
		.add(upub::model::activity::Column::Actor.eq(uid))
		.add(upub::model::object::Column::AttributedTo.eq(uid));

	let (limit, offset) = page.pagination();
	let items = upub::Query::feed(auth.my_id(), page.replies.unwrap_or(true))
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.into_model::<RichActivity>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/actors/{id}/inbox/page"), page, apb::Node::array(items))
}

pub async fn post(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	AuthIdentity(_auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>,
) -> crate::ApiResult<StatusCode> {
	// POSTing to user inboxes is effectively the same as POSTing to the main inbox
	super::super::inbox::post(State(ctx), AuthIdentity(_auth), Json(activity)).await
}
