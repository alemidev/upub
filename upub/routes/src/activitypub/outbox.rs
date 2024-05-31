use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition};
use upub::{server::auth::AuthIdentity, Context};

use crate::{activitypub::{CreationResult, Pagination}, builders::JsonLD};

pub async fn get(State(ctx): State<Context>) -> upub::Result<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/outbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> upub::Result<JsonLD<serde_json::Value>> {
	crate::builders::paginate(
		upub::url!(ctx, "/outbox/page"),
		Condition::all()
			.add(auth.filter_condition())
			.add(upub::model::actor::Column::Domain.eq(ctx.domain().to_string())),
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
) -> upub::Result<CreationResult> {
	// TODO administrative actions may be carried out against this outbox?
	Err(StatusCode::NOT_IMPLEMENTED.into())
}