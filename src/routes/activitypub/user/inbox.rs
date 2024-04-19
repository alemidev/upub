use axum::{extract::{Path, Query, State}, http::StatusCode, Json};

use sea_orm::{ColumnTrait, Order, QueryFilter, QueryOrder, QuerySelect};
use crate::{errors::UpubError, model::{self, addressing::EmbeddedActivity}, routes::activitypub::{jsonld::LD, JsonLD, Pagination}, server::{auth::{AuthIdentity, Identity}, Context}, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match auth {
		Identity::Anonymous => Err(StatusCode::FORBIDDEN),
		Identity::Remote(_) => Err(StatusCode::FORBIDDEN),
		Identity::Local(user) => if ctx.uid(id.clone()) == user {
			Ok(JsonLD(ctx.ap_collection(&url!(ctx, "/users/{id}/inbox"), None).ld_context()))
		} else {
			Err(StatusCode::FORBIDDEN)
		},
	}
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let Identity::Local(uid) = auth else {
		// local inbox is only for local users
		return Err(UpubError::forbidden());
	};
	if uid != ctx.uid(id.clone()) {
		return Err(UpubError::forbidden());
	}
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	let activities = model::addressing::Entity::find_activities()
		.filter(model::addressing::Column::Actor.eq(&uid))
		.order_by(model::addressing::Column::Published, Order::Desc)
		.offset(offset)
		.limit(limit)
		.into_model::<EmbeddedActivity>()
		.all(ctx.db())
		.await?;
	Ok(JsonLD(
		ctx.ap_collection_page(
			&url!(ctx, "/users/{id}/inbox/page"),
			offset, limit,
			activities
				.into_iter()
				.map(|x| x.into())
				.collect()
		).ld_context()
	))
}

pub async fn post(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	AuthIdentity(_auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>,
) -> Result<(), UpubError> {
	// POSTing to user inboxes is effectively the same as POSTing to the main inbox
	super::super::inbox::post(State(ctx), AuthIdentity(_auth), Json(activity)).await
}
