use axum::{extract::{Path, Query, State}, http::StatusCode, Json};

use sea_orm::{ColumnTrait, Condition, Order, QueryFilter, QueryOrder, QuerySelect};
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
	let uid = ctx.uid(id.clone());
	match auth {
		Identity::Anonymous => Err(StatusCode::FORBIDDEN.into()),
		Identity::Remote(_) => Err(StatusCode::FORBIDDEN.into()),
		Identity::Local(user) => if uid == user {
			let limit = page.batch.unwrap_or(20).min(50);
			let offset = page.offset.unwrap_or(0);
			match model::addressing::Entity::find_activities()
				.filter(Condition::all().add(model::addressing::Column::Actor.eq(&user)))
				.order_by(model::addressing::Column::Published, Order::Asc)
				.offset(offset)
				.limit(limit)
				.into_model::<EmbeddedActivity>()
				.all(ctx.db())
				.await
			{
				Ok(activities) => {
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
				},
				Err(e) => {
					tracing::error!("failed paginating user inbox for {id}: {e}");
					Err(StatusCode::INTERNAL_SERVER_ERROR.into())
				},
			}
		} else {
			Err(StatusCode::FORBIDDEN.into())
		},
	}
}

pub async fn post(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	Json(activity): Json<serde_json::Value>
) -> Result<(), UpubError> {
	// POSTing to user inboxes is effectively the same as POSTing to the main inbox
	super::super::inbox::post(State(ctx), Json(activity)).await
}
