use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter};
use upub::{model::{self, addressing::Event, attachment::BatchFillable}, Context};
use apb::LD;

use crate::{builders::JsonLD, AuthIdentity};

use super::TryFetch;

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<TryFetch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let aid = ctx.aid(&id);
	// if auth.is_local() && query.fetch && !ctx.is_local(&aid) {
	// 	let obj = ctx.fetch_activity(&aid).await?;
	// 	if obj.id != aid {
	// 		return Err(crate::ApiError::Redirect(obj.id));
	// 	}
	// }

	let row = model::addressing::Entity::find_addressed(auth.my_id())
		.filter(model::activity::Column::Id.eq(&aid))
		.filter(auth.filter_condition())
		.into_model::<Event>()
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?;

	let mut attachments = row.load_attachments_batch(ctx.db()).await?;
	let attach = attachments.remove(&row.internal());

	Ok(JsonLD(row.ap(attach).ld_context()))
}

