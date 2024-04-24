use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter};

use crate::{model, routes::activitypub::{JsonLD, Pagination}, server::{auth::AuthIdentity, Context}, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let oid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.oid(id.clone())
	};

	let count = model::addressing::Entity::find_addressed()
		.filter(auth.filter_condition())
		.filter(model::object::Column::InReplyTo.eq(oid))
		.count(ctx.db())
		.await?;

	crate::server::builders::collection(&url!(ctx, "/objects/{id}/replies"), Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let oid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.oid(id.clone())
	};

	crate::server::builders::paginate(
		url!(ctx, "/objects/{id}/replies/page"),
		Condition::all()
			.add(auth.filter_condition())
			.add(model::object::Column::InReplyTo.eq(oid)),
		ctx.db(),
		page
	)
		.await
}
