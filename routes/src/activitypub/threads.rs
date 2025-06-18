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

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let filter = Condition::all()
		.add(auth.filter_objects())
		.add(upub::model::object::Column::Audience.is_not_null())
		.add(upub::model::object::Column::Published.gte(chrono::Utc::now() - chrono::Duration::weeks(1)));
	let (limit, offset) = page.pagination();
	let feed_opts = upub::selector::QueryFeedOptions {
		my_id: auth.my_id(),
		with_replies: page.replies.unwrap_or(true),
		sort_by_likes: true,
		..Default::default()
	};
	let items = upub::Query::feed(feed_opts)
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
	crate::builders::collection_page(&upub::url!(ctx, "/threads/page"), page, apb::Node::array(items))
}

