use axum::{extract::{Path, State}, http::StatusCode, Json};
use sea_orm::EntityTrait;

use crate::{activitystream::Base, model::object, server::Context};


pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	match object::Entity::find_by_id(ctx.uri("objects", id)).one(ctx.db()).await {
		Ok(Some(object)) => Ok(Json(object.underlying_json_object())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for object: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}
