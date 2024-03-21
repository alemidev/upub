pub mod user;
pub mod object;
pub mod activity;
pub mod jsonld;
pub use jsonld::JsonLD;

use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{EntityTrait, IntoActiveModel};

use crate::{activitystream::{object::{activity::{Activity, ActivityType}, actor::{ActorMut, ActorType}, ObjectMut, ObjectType}, Base, BaseMut, BaseType, Node}, model, server::Context, url};

use self::jsonld::LD;

pub const PUBLIC_TARGET : &str = "https://www.w3.org/ns/activitystreams#Public";

pub fn split_id(id: &str) -> (String, String) {
	let clean = id
		.replace("http://", "")
		.replace("https://", "");
	let mut splits = clean.split('/');
	let first = splits.next().unwrap_or("");
	let last = splits.last().unwrap_or(first);
	(first.to_string(), last.to_string())
}

pub fn domain(domain: &str) -> String {
	domain
		.replace("http://", "")
		.replace("https://", "")
		.replace('/', "")
}


#[derive(Debug, serde::Deserialize)]
// TODO i don't really like how pleroma/mastodon do it actually, maybe change this?
pub struct Page {
	pub page: Option<bool>,
	pub max_id: Option<String>,
}

pub async fn view(State(ctx): State<Context>) -> Result<Json<serde_json::Value>, StatusCode> {
	Ok(Json(
		serde_json::Value::new_object()
			.set_actor_type(Some(ActorType::Application))
			.set_id(Some(&url!(ctx, "")))
			.set_name(Some("μpub"))
			.set_summary(Some("micro social network, federated"))
			.set_inbox(Node::link(url!(ctx, "/inbox")))
			.set_outbox(Node::link(url!(ctx, "/outbox")))
	))
}

pub async fn inbox(State(ctx) : State<Context>, Json(object): Json<serde_json::Value>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
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
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			model::activity::Entity::insert(activity_entity.into_active_model())
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(JsonLD(serde_json::Value::Null)) // TODO hmmmmmmmmmmm not the best value to return....
		},
		Some(BaseType::Object(ObjectType::Activity(_x))) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(_x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}

pub async fn outbox(State(_db): State<Context>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}
