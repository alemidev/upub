use axum::extract::{Query, State};
use upub::Context;

use crate::{AuthIdentity, builders::JsonLD};

use super::Pagination;


pub async fn get(
	State(ctx): State<Context>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/feed"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::paginate_objects(
		upub::url!(ctx, "/feed/page"),
		auth.filter_objects(),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}
