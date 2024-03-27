use axum::{extract::{Path, Query, State}, http::StatusCode};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, SelectColumns};

use crate::{activitypub::{jsonld::LD, JsonLD, Pagination}, activitystream::{object::collection::{page::CollectionPageMut, CollectionMut, CollectionType}, BaseMut, Node}, model, server::Context, url};

use model::relation::Column::{Following, Follower};

pub async fn get<const OUTGOING: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let follow___ = if OUTGOING { "following" } else { "followers" };
	let count = model::relation::Entity::find()
		.filter(Condition::all().add(if OUTGOING { Follower } else { Following }.eq(id.clone())))
		.count(ctx.db()).await.unwrap_or_else(|e| {
			tracing::error!("failed counting {follow___} for {id}: {e}");
			0
		});
	Ok(JsonLD(
		serde_json::Value::new_object()
			.set_id(Some(&format!("{}/users/{id}/{follow___}", ctx.base())))
			.set_collection_type(Some(CollectionType::OrderedCollection))
			.set_total_items(Some(count))
			.set_first(Node::link(format!("{}/users/{id}/{follow___}/page", ctx.base())))
			.ld_context()
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
		.filter(Condition::all().add(if OUTGOING { Follower } else { Following }.eq(id.clone())))
		.select_column(if OUTGOING { Following } else { Follower })
		.limit(limit)
		.offset(page.offset.unwrap_or(0))
		.all(ctx.db()).await
	{
		Err(e) => {
			tracing::error!("error queriying {follow___} for {id}: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
		Ok(following) => {
			Ok(JsonLD(
				serde_json::Value::new_object()
					.set_collection_type(Some(CollectionType::OrderedCollectionPage))
					.set_part_of(Node::link(url!(ctx, "/users/{id}/{follow___}")))
					.set_next(Node::link(url!(ctx, "/users/{id}/{follow___}/page?offset={}", offset+limit)))
					.set_ordered_items(Node::array(following.into_iter().map(|x| x.following).collect()))
					.ld_context()
			))
		},
	}
}
