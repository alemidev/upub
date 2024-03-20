use axum::{extract::{Path, State}, http::StatusCode, Json};
use sea_orm::EntityTrait;

use crate::{activitystream::Base, model::activity, server::Context};


pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	match activity::Entity::find_by_id(ctx.activity_uri(id)).one(ctx.db()).await {
		Ok(Some(activity)) => Ok(Json(activity.underlying_json_object())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for activity: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

