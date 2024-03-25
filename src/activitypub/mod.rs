pub mod user;
pub mod inbox;
pub mod outbox;
pub mod object;
pub mod activity;
pub mod well_known;

pub mod jsonld;
pub use jsonld::JsonLD;

use axum::{extract::State, http::StatusCode, Json};
use rand::Rng;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{activitystream::{object::{actor::{ActorMut, ActorType}, ObjectMut}, BaseMut}, model, server::Context, url};

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
					id: sea_orm::ActiveValue::NotSet,
					actor: sea_orm::ActiveValue::Set(x.id),
					session: sea_orm::ActiveValue::Set(token.clone()),
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
