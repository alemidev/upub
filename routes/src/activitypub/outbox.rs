use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition, QueryFilter, QueryOrder, QuerySelect, RelationTrait};
use upub::{selector::{RichActivity, RichFillable}, Context};

use crate::{activitypub::{CreationResult, Pagination}, AuthIdentity, builders::JsonLD};

pub async fn get(State(ctx): State<Context>) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(upub::url!(ctx, "/outbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let filter = Condition::all()
		.add(upub::model::addressing::Column::Actor.is_null())
		.add(upub::model::actor::Column::Domain.eq(ctx.domain().to_string()));
	
	let (limit, offset) = page.pagination();
	let items = upub::Query::feed(auth.my_id(), page.replies.unwrap_or(true))
		.join(sea_orm::JoinType::InnerJoin, upub::model::object::Relation::Actors.def())
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

	crate::builders::collection_page(&upub::url!(ctx, "/outbox/page"), page, apb::Node::array(items))
}

pub async fn post(
	State(_ctx): State<Context>,
	AuthIdentity(_auth): AuthIdentity,
	Json(_activity): Json<serde_json::Value>,
) -> crate::ApiResult<CreationResult> {
	// TODO administrative actions may be carried out against this outbox?
	Err(StatusCode::NOT_IMPLEMENTED.into())
}
