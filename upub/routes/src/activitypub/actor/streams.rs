use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition};

use upub::Context;

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/actors/{id}/streams"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::paginate_objects(
		upub::url!(ctx, "/actors/{id}/streams/page"),
		Condition::all()
			.add(auth.filter_objects())
			.add(upub::model::object::Column::AttributedTo.eq(ctx.uid(&id))),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}
