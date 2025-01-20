use axum::extract::{Path, Query, State};
use sea_orm::{Condition, ColumnTrait};

use upub::Context;

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(
		&upub::url!(ctx, "/actors/{id}/audience"),
		None,
	)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let filter = Condition::all()
		.add(auth.filter())
		.add(upub::model::object::Column::Audience.eq(ctx.uid(&id)));

	crate::builders::paginate_feed(
		upub::url!(ctx, "/actors/{id}/audience/page"),
		filter,
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}
