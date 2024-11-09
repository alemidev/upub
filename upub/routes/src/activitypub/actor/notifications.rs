use axum::extract::{Path, Query, State};
use sea_orm::{PaginatorTrait, QuerySelect};

use upub::{selector::RichNotification, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let Identity::Local { id: uid, internal } = &auth else {
		// notifications are only for local users
		return Err(crate::ApiError::forbidden());
	};
	if uid != &ctx.uid(&id) {
		return Err(crate::ApiError::forbidden());
	}

	let count = upub::Query::notifications(*internal, false)
		.count(ctx.db())
		.await?;

	crate::builders::collection(&upub::url!(ctx, "/actors/{id}/notifications"), Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let Identity::Local { id: uid, internal } = &auth else {
		// notifications are only for local users
		return Err(crate::ApiError::forbidden());
	};
	if uid != &ctx.uid(&id) {
		return Err(crate::ApiError::forbidden());
	}

	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let activities = upub::Query::notifications(*internal, true)
		.limit(limit)
		.offset(offset)
		.into_model::<RichNotification>()
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|x| x.ap())
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/actors/{id}/notifications/page"), offset, limit, activities)

}
