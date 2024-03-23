mod inbox;
pub use inbox::inbox;

mod outbox;
pub use outbox::outbox;

mod following;
pub use following::follow___;

use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{activitystream::Base, model::user, server::Context};

use super::{jsonld::LD, JsonLD};

pub async fn list(State(_db) : State<Arc<DatabaseConnection>>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	todo!()
}

pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match user::Entity::find_by_id(ctx.uid(id)).one(ctx.db()).await {
		Ok(Some(user)) => Ok(JsonLD(user.underlying_json_object().ld_context())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for user: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}


