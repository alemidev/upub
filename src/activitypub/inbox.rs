use axum::{extract::{Query, State}, http::StatusCode};
use sea_orm::{ColumnTrait, Condition, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect};

use crate::{activitystream::Node, auth::{AuthIdentity, Identity}, errors::UpubError, model, server::Context, url};

use super::{activity::ap_activity, jsonld::LD, JsonLD, Pagination, PUBLIC_TARGET};


pub async fn get(
	State(ctx): State<Context>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	Ok(JsonLD(ctx.ap_collection(&url!(ctx, "/inbox"), None).ld_context()))
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> Result<JsonLD<serde_json::Value>, UpubError> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	let mut condition = Condition::any()
		.add(model::addressing::Column::Actor.eq(PUBLIC_TARGET));
	if let Identity::Local(user) = auth {
		condition = condition
			.add(model::addressing::Column::Actor.eq(user));
	}
	let activities = model::addressing::Entity::find()
		.filter(condition)
		.order_by(model::addressing::Column::Published, Order::Asc)
		.find_also_related(model::activity::Entity)
		.limit(limit)
		.offset(offset)
		.all(ctx.db())
		.await?;
	Ok(JsonLD(
		ctx.ap_collection_page(
			&url!(ctx, "/inbox/page"),
			offset, limit,
			activities
				.into_iter()
				.filter_map(|(_, a)| Some(Node::object(ap_activity(a?))))
				.collect::<Vec<Node<serde_json::Value>>>()
		).ld_context()
	))
}
