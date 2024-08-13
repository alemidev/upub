use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter};
use upub::{model, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	let count = upub::Query::feed(auth.my_id(), true)
		.filter(auth.filter())
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

	let mut filter = Condition::any()
		.add(auth.filter());

	if let Identity::Local { ref id, .. } = auth {
		filter = filter.add(model::object::Column::AttributedTo.eq(id));
	}

	filter = Condition::all()
		.add(model::object::Column::Context.eq(context))
		.add(filter);


	crate::builders::paginate_feed(
		upub::url!(ctx, "/objects/{id}/context/page"),
		filter,
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}
