use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter, QuerySelect};
use upub::{model, selector::{RichFillable, RichObject}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	let count = upub::Query::objects(auth.my_id(), true)
		.filter(auth.filter_objects())
		.filter(model::object::Column::Context.eq(&context))
		.count(ctx.db())
		.await?;

	crate::builders::collection(upub::url!(ctx, "/objects/{id}/context"), Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(mut page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	let filter = Condition::all()
		.add(auth.filter_objects())
		.add(model::object::Column::Context.eq(context));

	page.replies = Some(true); // TODO ugly that we have to force set it this way...
	let (limit, offset) = page.pagination();

	let items = upub::Query::feed(auth.my_id(), page.replies.unwrap_or(true))
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.into_model::<RichObject>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/objects/{id}/context/page"), page, apb::Node::array(items))
}
