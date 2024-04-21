use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{model::{self, addressing::WrappedObject}, routes::activitypub::{jsonld::LD, JsonLD, Pagination}, server::{auth::AuthIdentity, Context}, url};

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

	let count = model::addressing::Entity::find_objects()
		.filter(auth.filter_condition())
		.filter(model::object::Column::InReplyTo.eq(oid))
		.count(ctx.db())
		.await?;

	Ok(JsonLD(ctx.ap_collection(&url!(ctx, "/objects/{id}/replies"), Some(count)).ld_context()))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let oid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.oid(id.clone())
	};

	let items = model::addressing::Entity::find_objects()
		.filter(auth.filter_condition())
		.filter(model::object::Column::InReplyTo.eq(oid))
		// TODO also limit to only local activities
		.order_by(model::addressing::Column::Published, Order::Desc)
		.limit(limit)
		.offset(offset)
		.into_model::<WrappedObject>()
		.all(ctx.db())
		.await?;

	Ok(JsonLD(
		ctx.ap_collection_page(
			&url!(ctx, "/objects/{id}/replies/page"),
			offset, limit,
			items
				.into_iter()
				.map(|x| x.object.ap())
				.collect()
		).ld_context()
	))
}
