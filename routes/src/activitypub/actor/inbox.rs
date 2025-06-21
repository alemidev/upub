use axum::{http::StatusCode, extract::{Path, Query, State}, Json};
use sea_orm::{ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect};

use upub::{selector::{RichActivity, RichFillable}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	auth.check(&ctx.uid(&id), true)?;

	crate::builders::collection(upub::url!(ctx, "/actors/{id}/inbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(&id);
	let internal = auth.check(&uid, true)?;

	let filter = Condition::any()
		.add(upub::model::addressing::Column::Actor.eq(internal))
		.add(upub::model::activity::Column::Actor.eq(&uid))
		.add(upub::model::object::Column::AttributedTo.eq(&uid));

	let (limit, offset) = page.pagination();
	let items = upub::Query::feed(upub::query_feed_opts!(auth.my_id(), page.replies()))
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
