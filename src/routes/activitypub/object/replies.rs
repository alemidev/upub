use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter};

use crate::{model, routes::activitypub::{JsonLD, Pagination}, server::{auth::AuthIdentity, Context}, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let replies_id = url!(ctx, "/objects/{id}/replies");
	let oid = ctx.uri("objects", id);

	let count = model::addressing::Entity::find_addressed()
		.filter(auth.filter_condition())
		.filter(model::object::Column::InReplyTo.eq(oid))
		.count(ctx.db())
		.await?;

	crate::server::builders::collection(&replies_id, Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let page_id = url!(ctx, "/objects/{id}/replies/page");
	let oid = ctx.uri("objects", id);

	crate::server::builders::paginate(
		page_id,
		Condition::all()
			.add(auth.filter_condition())
			.add(model::object::Column::InReplyTo.eq(oid)),
		ctx.db(),
		page
	)
		.await
}
