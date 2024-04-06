pub mod user;
pub mod inbox;
pub mod outbox;
pub mod object;
pub mod activity;
pub mod well_known;

pub mod jsonld;
pub use jsonld::JsonLD;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand::Rng;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use apb::{PublicKeyMut, ActorMut, ActorType, Link, Object, ObjectMut, BaseMut, Node};
use crate::{model, server::Context, url};

use self::jsonld::LD;

pub trait Addressed : Object {
	fn addressed(&self) -> Vec<String>;
}

impl Addressed for serde_json::Value {
	fn addressed(&self) -> Vec<String> {
		let mut to : Vec<String> = self.to().map(|x| x.href().to_string()).collect();
		to.append(&mut self.bto().map(|x| x.href().to_string()).collect());
		to.append(&mut self.cc().map(|x| x.href().to_string()).collect());
		to.append(&mut self.bcc().map(|x| x.href().to_string()).collect());
		to
	}
}

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
	pub offset: Option<u64>,
	pub batch: Option<u64>,
}

pub struct CreationResult(pub String);
impl IntoResponse for CreationResult {
	fn into_response(self) -> axum::response::Response {
		(
			StatusCode::CREATED,
			[("Location", self.0.as_str())]
		)
			.into_response()
	}
}

pub async fn view(State(ctx): State<Context>) -> Result<Json<serde_json::Value>, StatusCode> {
	Ok(Json(
		serde_json::Value::new_object()
			.set_id(Some(&url!(ctx, "")))
			.set_actor_type(Some(ActorType::Application))
			.set_name(Some("μpub"))
			.set_summary(Some("micro social network, federated"))
			.set_published(Some(ctx.app().created))
			.set_public_key(Node::object(
				serde_json::Value::new_object()
					.set_id(Some(&url!(ctx, "#main-key")))
					.set_owner(Some(&url!(ctx, "")))
					.set_public_key_pem(&ctx.app().public_key)
			))
			.ld_context()
	))
}


#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoginForm {
	email: String,
	password: String,
}

pub async fn auth(State(ctx): State<Context>, Json(login): Json<LoginForm>) -> Result<Json<serde_json::Value>, StatusCode> {
	// TODO salt the pwd
	match model::credential::Entity::find()
		.filter(Condition::all()
			.add(model::credential::Column::Email.eq(login.email))
			.add(model::credential::Column::Password.eq(sha256::digest(login.password)))
		)
		.one(ctx.db())
		.await
	{
		Ok(Some(x)) => {
			// TODO should probably use crypto-safe rng
			let token : String = rand::thread_rng()
				.sample_iter(&rand::distributions::Alphanumeric)
				.take(128)
				.map(char::from)
				.collect();
			model::session::Entity::insert(
				model::session::ActiveModel {
					id: sea_orm::ActiveValue::Set(token.clone()),
					actor: sea_orm::ActiveValue::Set(x.id),
					expires: sea_orm::ActiveValue::Set(chrono::Utc::now() + std::time::Duration::from_secs(3600 * 6)),
				}
			)
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(Json(serde_json::Value::String(token)))
		},
		Ok(None) => Err(StatusCode::UNAUTHORIZED),
		Err(e) => {
			tracing::error!("error querying db for user credentials: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}
