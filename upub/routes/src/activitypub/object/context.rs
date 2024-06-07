use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter};
use upub::{model, Context};

use crate::{AuthIdentity, builders::JsonLD, activitypub::Pagination};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	let count = model::addressing::Entity::find_addressed(auth.my_id())
		.filter(auth.filter_condition())
		.filter(model::object::Column::Context.eq(&context))
		.count(ctx.db())
		.await?;

	crate::builders::collection(&upub::url!(ctx, "/objects/{id}/context"), Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	crate::builders::paginate(
		upub::url!(ctx, "/objects/{id}/context/page"),
		Condition::all()
			.add(auth.filter_condition())
			.add(model::object::Column::Context.eq(context)),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}