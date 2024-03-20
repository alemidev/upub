use std::{ops::Deref, sync::Arc};

use axum::{extract::{Path, State}, http::StatusCode, Json};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{activitystream::Base, model::activity};


pub async fn view(State(db) : State<Arc<DatabaseConnection>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	let uri = format!("http://localhost:3000/activities/{id}");
	match activity::Entity::find_by_id(uri).one(db.deref()).await {
		Ok(Some(activity)) => Ok(Json(activity.underlying_json_object())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for activity: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

