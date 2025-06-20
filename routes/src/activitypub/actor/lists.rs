use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};

use upub::{model, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(&id);

	if !auth.is(&uid) {
		return Err(crate::ApiError::unauthorized());
	}

	let lists_count = model::list::Entity::find()
		.filter(model::list::Column::AttributedTo.eq(&uid))
		.count(ctx.db())
		.await?;

	crate::builders::collection(upub::url!(ctx, "/actors/{id}/lists"), Some(lists_count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let (limit, offset) = page.pagination();
	let uid = ctx.uid(&id);

	if !auth.is(&uid) {
		return Err(crate::ApiError::unauthorized());
	}

	let lists = model::list::Entity::find()
		.filter(model::list::Column::AttributedTo.eq(&uid))
		.limit(limit)
		.offset(offset)
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|x| ctx.ap(x))
		.collect();

	crate::builders::collection_page(
		&upub::url!(ctx, "/actors/{id}/lists/page"),
		page,
		apb::Node::array(lists),
	)
}

