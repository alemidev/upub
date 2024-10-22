use axum::extract::{Path, Query, State};
use sea_orm::{QueryFilter, QuerySelect, ColumnTrait};

use upub::{selector::{BatchFillable, RichActivity}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(
		&upub::url!(ctx, "/tags/{id}"),
		None,
	)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let objects = upub::Query::hashtags()
		.filter(auth.filter())
		.filter(upub::model::hashtag::Column::Name.eq(&id))
		.limit(limit)
		.offset(offset)
		.into_model::<RichActivity>()
		.all(ctx.db())
		.await?
		.with_batched::<upub::model::attachment::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::mention::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::hashtag::Entity>(ctx.db())
		.await?
		.into_iter()
		.map(|x| x.ap())
		.collect();

	crate::builders::collection_page(
		&upub::url!(ctx, "/tags/{id}/page"),
		offset,
		limit,
		objects,
	)

}
