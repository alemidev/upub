use axum::extract::{Query, State};
use sea_orm::{ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect};
use upub::{selector::{RichActivity, RichFillable}, Context};

use crate::{AuthIdentity, builders::JsonLD};

use super::Pagination;


pub async fn get(
	State(ctx): State<Context>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(upub::url!(ctx, "/threads"), None)
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct PaginationWithDays {
	#[serde(flatten)]
	page: Pagination,

	days: Option<i64>,
	skip: Option<i64>, // TODO is this a bad name?
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<PaginationWithDays>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let days = query.days.unwrap_or(30);
	let skip = query.skip.unwrap_or(0);
	let filter = Condition::all()
		.add(auth.filter_objects())
		.add(upub::model::object::Column::Audience.is_not_null())
		.add(upub::model::object::Column::Published.lte(chrono::Utc::now() - chrono::Duration::days(skip)))
		.add(upub::model::object::Column::Published.gte(chrono::Utc::now() - chrono::Duration::days(days)));
	let (limit, offset) = query.page.pagination();
	let items = upub::Query::feed(upub::query_feed_opts!(auth.my_id(), query.page.replies.unwrap_or(true), true))
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.order_by_desc(upub::model::object::Column::Likes)
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
	crate::builders::collection_page(&upub::url!(ctx, "/threads/page?days={days}&skip={skip}"), query.page, apb::Node::array(items))
}

