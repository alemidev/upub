use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{sea_query::IntoCondition, ColumnTrait};
use upub::Context;

use crate::{activitypub::{CreationResult, Pagination}, AuthIdentity, builders::JsonLD};

pub async fn get(State(ctx): State<Context>) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/outbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::paginate_activities(
		upub::url!(ctx, "/outbox/page"),
		upub::model::actor::Column::Domain.eq(ctx.domain().to_string()).into_condition(),
		ctx.db(),
		page,
		auth.my_id(),
		true,
	)
		.await
}

pub async fn post(
	State(_ctx): State<Context>,
	AuthIdentity(_auth): AuthIdentity,
	Json(_activity): Json<serde_json::Value>,
) -> crate::ApiResult<CreationResult> {
	// TODO administrative actions may be carried out against this outbox?
	Err(StatusCode::NOT_IMPLEMENTED.into())
}
