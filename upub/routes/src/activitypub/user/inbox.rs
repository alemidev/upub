use axum::{extract::{Path, Query, State}, http::StatusCode, Json};

use sea_orm::{ColumnTrait, Condition};
use upub::{model, server::auth::{AuthIdentity, Identity}, Context};

use crate::{activitypub::Pagination, builders::JsonLD};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> upub::Result<JsonLD<serde_json::Value>> {
	match auth {
		Identity::Anonymous => Err(StatusCode::FORBIDDEN.into()),
		Identity::Remote { .. } => Err(StatusCode::FORBIDDEN.into()),
		Identity::Local { id: user, .. } => if ctx.uid(&id) == user {
			crate::builders::collection(&upub::url!(ctx, "/actors/{id}/inbox"), None)
		} else {
			Err(StatusCode::FORBIDDEN.into())
		},
	}
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> upub::Result<JsonLD<serde_json::Value>> {
	let Identity::Local { id: uid, internal } = &auth else {
		// local inbox is only for local users
		return Err(upub::Error::forbidden());
	};
	if uid != &ctx.uid(&id) {
		return Err(upub::Error::forbidden());
	}

	crate::builders::paginate(
		upub::url!(ctx, "/actors/{id}/inbox/page"),
		Condition::any()
			.add(model::addressing::Column::Actor.eq(*internal))
			.add(model::object::Column::AttributedTo.eq(uid))
			.add(model::activity::Column::Actor.eq(uid)),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}

pub async fn post(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	AuthIdentity(_auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>,
) -> Result<(), upub::Error> {
	// POSTing to user inboxes is effectively the same as POSTing to the main inbox
	super::super::inbox::post(State(ctx), AuthIdentity(_auth), Json(activity)).await
}
