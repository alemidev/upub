use axum::{extract::{Query, State}, http::StatusCode};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::{activitystream::{object::collection::{page::CollectionPageMut, CollectionMut, CollectionType}, BaseMut, Node}, model, server::Context, url};

use super::{activity::ap_activity, jsonld::LD, JsonLD, Pagination, PUBLIC_TARGET};


pub async fn get(State(ctx) : State<Context>, Query(page): Query<Pagination>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(100);
	let offset = page.offset.unwrap_or(0);
	if let Some(true) = page.page {
		match model::addressing::Entity::find()
			.filter(Condition::all().add(model::addressing::Column::Actor.eq(PUBLIC_TARGET)))
			.order_by(model::addressing::Column::Published, sea_orm::Order::Desc)
			.find_also_related(model::activity::Entity) // TODO join also with objects
			.limit(limit)
			.offset(offset)
			.all(ctx.db())
			.await
		{
			Ok(x) => Ok(JsonLD(serde_json::Value::new_object()
				.set_id(Some(&url!(ctx, "/inbox")))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_part_of(Node::link(url!(ctx, "/inbox")))
				.set_next(Node::link(url!(ctx, "/inbox?page=true&offset={}", offset+limit)))
				.set_ordered_items(Node::array(
						x.into_iter()
							.filter_map(|(_, a)| Some(ap_activity(a?)))
							.collect()
				))
				.ld_context()
			)),
			Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
		}
	} else {
		Ok(JsonLD(serde_json::Value::new_object()
			.set_id(Some(&url!(ctx, "/inbox")))
			.set_collection_type(Some(CollectionType::OrderedCollection))
			.set_first(Node::link(url!(ctx, "/inbox?page=true")))
			.ld_context()
		))
	}
}

