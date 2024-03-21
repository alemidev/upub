use axum::{extract::{Path, State}, http::StatusCode};
use sea_orm::EntityTrait;
use crate::{activitystream::{object::activity::ActivityMut, Base, Node}, model::{activity, object}, server::Context};

use super::{jsonld::LD, JsonLD};


pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match activity::Entity::find_by_id(ctx.aid(id))
		.find_also_related(object::Entity)
		.one(ctx.db())
		.await
	{
		Ok(Some((activity, object))) => Ok(JsonLD(
			activity
				.underlying_json_object()
				.set_object(Node::maybe_object(object))
				.ld_context()
		)),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for activity: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

