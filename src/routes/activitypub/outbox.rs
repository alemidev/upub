use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{Order, QueryFilter, QueryOrder, QuerySelect};

use crate::{errors::UpubError, model::{self, addressing::EmbeddedActivity}, routes::activitypub::{jsonld::LD, CreationResult, JsonLD, Pagination}, server::{auth::AuthIdentity, Context}, url};

pub async fn get(State(ctx): State<Context>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	Ok(JsonLD(ctx.ap_collection(&url!(ctx, "/outbox"), None).ld_context()))
}

pub async fn page(
	State(ctx): State<Context>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	match model::addressing::Entity::find_activities()
		.filter(auth.filter_condition())
		// TODO also limit to only local activities
		.order_by(model::addressing::Column::Published, Order::Asc)
		.limit(limit)
		.offset(offset)
		.into_model::<EmbeddedActivity>()
		.all(ctx.db()).await
	{
		Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
		Ok(items) => {
			Ok(JsonLD(
				ctx.ap_collection_page(
					&url!(ctx, "/outbox/page"),
					offset, limit,
					items
						.into_iter()
						.map(|x| x.into())
						.collect()
				).ld_context()
			))
		},
	}
}

pub async fn post(
	State(_ctx): State<Context>,
	AuthIdentity(_auth): AuthIdentity,
	Json(_activity): Json<serde_json::Value>,
) -> Result<CreationResult, UpubError> {
	// TODO administrative actions may be carried out against this outbox?
	Err(StatusCode::NOT_IMPLEMENTED.into())
}