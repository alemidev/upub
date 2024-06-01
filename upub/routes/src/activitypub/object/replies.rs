use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter};
use upub::{model, Context};

use crate::{activitypub::{Pagination, TryFetch}, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(q): Query<TryFetch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let replies_id = upub::url!(ctx, "/objects/{id}/replies");
	let oid = ctx.oid(&id);

	// if auth.is_local() && q.fetch {
	// 	ctx.fetch_thread(&oid).await?;
	// }

	let count = model::addressing::Entity::find_addressed(auth.my_id())
		.filter(auth.filter_condition())
		.filter(model::object::Column::InReplyTo.eq(oid))
		.count(ctx.db())
		.await?;

	crate::builders::collection(&replies_id, Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let page_id = upub::url!(ctx, "/objects/{id}/replies/page");
	let oid = ctx.oid(&id);

	crate::builders::paginate(
		page_id,
		Condition::all()
			.add(auth.filter_condition())
			.add(model::object::Column::InReplyTo.eq(oid)),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}
