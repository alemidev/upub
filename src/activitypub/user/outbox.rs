use axum::{extract::{Path, Query, State}, http::StatusCode};
use sea_orm::{EntityTrait, Order, QueryOrder, QuerySelect};

use crate::{activitypub::{jsonld::LD, JsonLD, Pagination}, activitystream::{object::{activity::ActivityMut, collection::{page::CollectionPageMut, CollectionMut, CollectionType}}, Base, BaseMut, Node}, model::{activity, object}, server::Context, url};

pub async fn outbox(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	if let Some(true) = page.page {
		match activity::Entity::find()
			.find_also_related(object::Entity)
			.order_by(activity::Column::Published, Order::Desc)
			.limit(limit)
			.offset(offset)
			.all(ctx.db()).await
		{
			Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
			Ok(items) => {
				let next = ctx.id(items.last().map(|(a, _o)| a.id.as_str()).unwrap_or("").to_string());
				let items = items
					.into_iter()
					.map(|(a, o)| a.underlying_json_object().set_object(Node::maybe_object(o)))
					.collect();
				Ok(JsonLD(
					serde_json::Value::new_object()
						// TODO set id, calculate uri from given args
						.set_collection_type(Some(CollectionType::OrderedCollectionPage))
						.set_part_of(Node::link(url!(ctx, "/users/{id}/outbox")))
						.set_next(Node::link(url!(ctx, "/users/{id}/outbox?page=true&max_id={next}")))
						.set_ordered_items(Node::array(items))
						.ld_context()
				))
			},
		}

	} else {
		Ok(JsonLD(
			serde_json::Value::new_object()
				.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_first(Node::link(url!(ctx, "/users/{id}/outbox?page=true")))
				.ld_context()
		))
	}
}

