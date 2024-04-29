use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter};
use crate::{errors::UpubError, model::{self, addressing::Event, attachment::BatchFillable}, server::{auth::AuthIdentity, fetcher::Fetcher, Context}};

use super::{jsonld::LD, JsonLD, TryFetch};

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<TryFetch>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let aid = ctx.uri("activities", id);
	if auth.is_local() && query.fetch && !ctx.is_local(&aid) {
		ctx.fetch_activity(&aid).await?;
	}

	let row = model::addressing::Entity::find_addressed()
		.filter(model::activity::Column::Id.eq(&aid))
		.filter(auth.filter_condition())
		.into_model::<Event>()
		.one(ctx.db())
		.await?
		.ok_or_else(UpubError::not_found)?;

	let mut attachments = row.load_attachments_batch(ctx.db()).await?;
	let attach = attachments.remove(row.id());

	Ok(JsonLD(row.ap(attach).ld_context()))
}

