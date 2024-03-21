use axum::{extract::{Path, State}, http::StatusCode};
use sea_orm::EntityTrait;

use crate::{activitystream::Base, model::object, server::Context};

use super::{jsonld::LD, JsonLD};


pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match object::Entity::find_by_id(ctx.oid(id)).one(ctx.db()).await {
		Ok(Some(object)) => Ok(JsonLD(object.underlying_json_object().ld_context())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for object: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}
