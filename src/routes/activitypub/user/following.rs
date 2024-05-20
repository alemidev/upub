use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, SelectColumns};

use crate::{routes::activitypub::{JsonLD, Pagination}, model, server::Context, url};

use model::relation::Column::{Following, Follower};

pub async fn get<const OUTGOING: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let follow___ = if OUTGOING { "following" } else { "followers" };
	let count = model::relation::Entity::find()
		.filter(if OUTGOING { Follower } else { Following }.eq(ctx.uid(&id)))
		.count(ctx.db()).await.unwrap_or_else(|e| {
			tracing::error!("failed counting {follow___} for {id}: {e}");
			0
		});

	crate::server::builders::collection(&url!(ctx, "/users/{id}/{follow___}"), Some(count))
}

pub async fn page<const OUTGOING: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let follow___ = if OUTGOING { "following" } else { "followers" };
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let following = model::relation::Entity::find()
		.filter(if OUTGOING { Follower } else { Following }.eq(ctx.uid(&id)))
		.select_only()
		.select_column(if OUTGOING { Following } else { Follower })
		.limit(limit)
		.offset(page.offset.unwrap_or(0))
		.into_tuple::<String>()
		.all(ctx.db())
		.await?;

	crate::server::builders::collection_page(
		&url!(ctx, "/users/{id}/{follow___}/page"),
		offset, limit,
		following.into_iter().map(serde_json::Value::String).collect()
	)
}
