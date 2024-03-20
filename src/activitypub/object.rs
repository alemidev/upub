use std::{ops::Deref, sync::Arc};

use axum::{extract::{Path, State}, http::StatusCode, Json};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{activitystream::Base, model::object};


pub async fn view(State(db) : State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	let uri = format!("http://localhost:3000/objects/{id}");
	match object::Entity::find_by_id(uri).one(db.deref()).await {
		Ok(Some(object)) => Ok(Json(object.underlying_json_object())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for object: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}
