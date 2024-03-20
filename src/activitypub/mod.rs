pub mod user;
pub mod object;
pub mod activity;

use std::{ops::Deref, sync::Arc};

use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};

use crate::{activitystream::{object::{ObjectType, activity::{Activity, ActivityType}}, Base, BaseType, Node}, model};


pub fn uri_id(entity: &str, id: String) -> String {
	if id.starts_with("http") { id } else { format!("http://localhost:3000/{entity}/{id}") }
}

pub fn id_uri(id: &str) -> &str {
	id.split('/').last().unwrap_or("")
}

#[derive(Debug, serde::Deserialize)]
// TODO i don't really like how pleroma/mastodon do it actually, maybe change this?
pub struct Page {
	pub page: Option<bool>,
	pub max_id: Option<String>,
}

pub async fn inbox(State(db) : State<Arc<DatabaseConnection>>, Json(object): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, StatusCode> {
	match object.base_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(BaseType::Link(_x)) => Err(StatusCode::UNPROCESSABLE_ENTITY), // we could but not yet
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => Err(StatusCode::UNPROCESSABLE_ENTITY),
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
			let Ok(activity_entity) = model::activity::Model::new(&object) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Node::Object(obj) = object.object() else {
				// TODO we could process non-embedded activities or arrays but im lazy rn
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Ok(obj_entity) = model::object::Model::new(&*obj) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			model::object::Entity::insert(obj_entity.into_active_model())
				.exec(db.deref())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			model::activity::Entity::insert(activity_entity.into_active_model())
				.exec(db.deref())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(Json(serde_json::Value::Null)) // TODO hmmmmmmmmmmm not the best value to return....
		},
		Some(BaseType::Object(ObjectType::Activity(_x))) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(_x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}

pub async fn outbox(State(_db): State<Arc<DatabaseConnection>>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}
