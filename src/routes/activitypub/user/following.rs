use axum::{extract::{Path, Query, State}, http::StatusCode};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, SelectColumns};

use crate::{routes::activitypub::{jsonld::LD, JsonLD, Pagination}, model, server::Context, url};

use model::relation::Column::{Following, Follower};

pub async fn get<const OUTGOING: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let follow___ = if OUTGOING { "following" } else { "followers" };
	let count = model::relation::Entity::find()
		.filter(if OUTGOING { Follower } else { Following }.eq(ctx.uid(id.clone())))
		.count(ctx.db()).await.unwrap_or_else(|e| {
			tracing::error!("failed counting {follow___} for {id}: {e}");
			0
		});
	Ok(JsonLD(
		ctx.ap_collection(
			&url!(ctx, "/users/{id}/{follow___}"),
			Some(count)
		).ld_context()
	))
}

pub async fn page<const OUTGOING: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let follow___ = if OUTGOING { "following" } else { "followers" };
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	match model::relation::Entity::find()
		.filter(if OUTGOING { Follower } else { Following }.eq(ctx.uid(id.clone())))
		.select_only()
		.select_column(if OUTGOING { Following } else { Follower })
		.limit(limit)
		.offset(page.offset.unwrap_or(0))
		.into_tuple::<String>()
		.all(ctx.db()).await
	{
		Err(e) => {
			tracing::error!("error queriying {follow___} for {id}: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
		Ok(following) => {
			Ok(JsonLD(
				ctx.ap_collection_page(
					&url!(ctx, "/users/{id}/{follow___}"),
					offset,
					limit,
					following.into_iter().map(serde_json::Value::String).collect()
				).ld_context()
			))
		},
	}
}
