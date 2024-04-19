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
	let aid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.aid(id.clone())
	};
	if auth.is_local() && query.fetch && !ctx.is_local(&aid) {
		ctx.fetch_activity(&aid).await?;
	}

	match model::addressing::Entity::find_activities()
		.filter(model::activity::Column::Id.eq(&aid))
		.filter(auth.filter_condition())
		.into_model::<EmbeddedActivity>()
		.one(ctx.db())
		.await?
	{
		Some(activity) => Ok(JsonLD(serde_json::Value::from(activity).ld_context())),
		None => Err(UpubError::not_found()),
	}
}

