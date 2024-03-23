use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, SelectColumns};

use crate::{activitypub::{JsonLD, Pagination}, activitystream::{object::{activity::{Activity, ActivityMut, ActivityType}, collection::{page::CollectionPageMut, CollectionMut, CollectionType}, ObjectType}, Base, BaseMut, BaseType, Node}, errors::LoggableError, model::{self, activity, object, user}, server::Context, url};

pub async fn following<const out: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let follow = if out { "following" } else { "followers" };
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	if let Some(true) = page.page {
		match model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Follower.eq(id.clone())))
			.select_column(model::relation::Column::Following)
			.limit(limit) // TODO allow customizing, with boundaries
			.offset(page.offset.unwrap_or(0))
			.all(ctx.db()).await
		{
			Err(e) => {
				tracing::error!("error queriying who {id} is following: {e}");
				Err(StatusCode::INTERNAL_SERVER_ERROR)
			},
			Ok(following) => {
				Ok(JsonLD(
					serde_json::Value::new_object()
						.set_collection_type(Some(CollectionType::OrderedCollectionPage))
						.set_part_of(Node::link(url!(ctx, "/users/{id}/following")))
						.set_next(Node::link(url!(ctx, "/users/{id}/following?page=true&offset={}", offset+limit)))
						.set_ordered_items(Node::array(following.into_iter().map(|x| x.following).collect()))
						.ld_context()
				))
			},
		}
	} else {
		let count = model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Follower.eq(id.clone())))
			.count(ctx.db()).await.unwrap_or_else(|e| {
				tracing::error!("failed counting following for {id}: {e}");
				0
			});
		Ok(JsonLD(
			serde_json::Value::new_object()
				.set_id(Some(&format!("{}/users/{}/following", ctx.base(), id)))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_total_items(Some(count))
				.set_first(Node::link(format!("{}/users/{}/following?page=true", ctx.base(), id)))
				.ld_context()
		))
	}
}














pub async fn followers<const from: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	if let Some(true) = page.page {
		match model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Following.eq(id.clone())))
			.select_column(model::relation::Column::Follower)
			.limit(limit) // TODO allow customizing, with boundaries
			.offset(page.offset.unwrap_or(0))
			.all(ctx.db()).await
		{
			Err(e) => {
				tracing::error!("error queriying who {id} is following: {e}");
				Err(StatusCode::INTERNAL_SERVER_ERROR)
			},
			Ok(following) => {
				Ok(JsonLD(
					serde_json::Value::new_object()
						.set_collection_type(Some(CollectionType::OrderedCollectionPage))
						.set_part_of(Node::link(url!(ctx, "/users/{id}/followers")))
						.set_next(Node::link(url!(ctx, "/users/{id}/followers?page=true&offset={}", offset+limit)))
						.set_ordered_items(Node::array(following.into_iter().map(|x| x.follower).collect()))
						.ld_context()
				))
			},
		}
	} else {
		let count = model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Following.eq(id.clone())))
			.count(ctx.db()).await.unwrap_or_else(|e| {
				tracing::error!("failed counting followers for {id}: {e}");
				0
			});
		Ok(JsonLD(
			serde_json::Value::new_object()
				.set_id(Some(&format!("{}/users/{}/followers", ctx.base(), id)))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_total_items(Some(count))
				.set_first(Node::link(format!("{}/users/{}/followers?page=true", ctx.base(), id)))
				.ld_context()
		))
	}
}
