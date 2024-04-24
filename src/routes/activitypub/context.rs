use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter};

use crate::{model, routes::activitypub::{JsonLD, Pagination}, server::{auth::AuthIdentity, Context}, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let context = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		url!(ctx, "/context/{id}")
	};

	let count = model::addressing::Entity::find_addressed()
		.filter(auth.filter_condition())
		.filter(model::object::Column::Context.eq(context))
		.count(ctx.db())
		.await?;

	crate::server::builders::collection(&url!(ctx, "/context/{id}"), Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let context = if id.starts_with('+') {
		id.replacen('+', "https://", 1).replace('@', "/")
	} else if id.starts_with("tag:") {
		id.clone()
	} else {
		url!(ctx, "/context/{id}") // TODO need a better way to figure out which ones are our contexts
	};

	crate::server::builders::paginate(
		url!(ctx, "/context/{id}/page"),
		Condition::all()
			.add(auth.filter_condition())
			.add(model::object::Column::Context.eq(context)),
		ctx.db(),
		page
	)
		.await
}
