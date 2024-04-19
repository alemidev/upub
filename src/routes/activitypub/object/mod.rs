pub mod replies;

use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter};

use crate::{errors::UpubError, model::{self, addressing::EmbeddedActivity}, server::{auth::AuthIdentity, fetcher::Fetcher, Context}};

use super::{jsonld::LD, JsonLD, TryFetch};

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<TryFetch>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let oid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.oid(id.clone())
	};
	match model::addressing::Entity::find_activities()
		.filter(model::object::Column::Id.eq(&oid))
		.filter(auth.filter_condition())
		.into_model::<EmbeddedActivity>()
		.one(ctx.db())
		.await?
	{
		Some(EmbeddedActivity { activity: _, object: Some(object) }) => Ok(JsonLD(object.ap().ld_context())),
		Some(EmbeddedActivity { activity: _, object: None }) => Err(UpubError::not_found()),
		None => if auth.is_local() && query.fetch && !ctx.is_local(&oid) {
			Ok(JsonLD(ctx.fetch_object(&oid).await?.ap().ld_context()))
		} else {
			Err(UpubError::not_found())
		},
	}
}
