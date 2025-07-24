use axum::extract::{Query, State};
use sea_orm::{ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect};
use upub::{selector::{RichActivity, RichFillable}, Context};

use crate::{AuthIdentity, builders::JsonLD};

use super::Pagination;


pub async fn get(
	State(ctx): State<Context>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(upub::url!(ctx, "/articles"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let filter = Condition::all()
		.add(auth.filter_objects())
		.add(upub::model::object::Column::ObjectType.eq(apb::ObjectType::Article));
	let (limit, offset) = query.pagination();
	let items = upub::Query::feed(upub::query_feed_opts!(auth.my_id(), query.replies()))
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.order_by_desc(upub::model::addressing::Column::Published)
		.order_by_desc(upub::model::activity::Column::Internal)
		.into_model::<RichActivity>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();
	crate::builders::collection_page(&upub::url!(ctx, "/articles/page"), query, apb::Node::array(items))
}

