use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use upub::{model, selector::{BatchFillable, RichActivity}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	let count = upub::Query::objects(auth.my_id())
		.filter(auth.filter())
		.filter(model::object::Column::Context.eq(&context))
		.count(ctx.db())
		.await?;

	crate::builders::collection(&upub::url!(ctx, "/objects/{id}/context"), Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let items = upub::Query::objects(auth.my_id())
		.filter(auth.filter())
		.filter(model::object::Column::Context.eq(context))
		.order_by(model::object::Column::Published, Order::Desc)
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
		.await?;

	let items : Vec<serde_json::Value> = items
		.into_iter()
		.map(|item| item.ap())
		.collect();

	crate::builders::collection_page(&id, offset, limit, items, true)
}
