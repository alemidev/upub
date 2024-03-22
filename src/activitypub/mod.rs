pub mod user;
pub mod object;
pub mod activity;
pub mod well_known;

pub mod jsonld;
pub use jsonld::JsonLD;

use axum::{extract::State, http::StatusCode, Json};

use crate::{activitystream::{object::{actor::{ActorMut, ActorType}, ObjectMut}, BaseMut}, server::Context, url};

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
pub struct Pagination {
	pub page: Option<bool>,
	pub offset: Option<u64>,
	pub batch: Option<u64>,
}

pub async fn view(State(ctx): State<Context>) -> Result<Json<serde_json::Value>, StatusCode> {
	Ok(Json(
		serde_json::Value::new_object()
			.set_actor_type(Some(ActorType::Application))
			.set_id(Some(&url!(ctx, "")))
			.set_name(Some("μpub"))
			.set_summary(Some("micro social network, federated"))
			// .set_inbox(Node::link(url!(ctx, "/inbox")))
			// .set_outbox(Node::link(url!(ctx, "/outbox")))
			.ld_context()
	))
}

pub async fn inbox(State(_ctx) : State<Context>, Json(_object): Json<serde_json::Value>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	todo!()
}

pub async fn outbox(State(_db): State<Context>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}
