use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, QueryFilter, TransactionTrait};
use upub::{model, selector::{RichActivity, RichFillable}, traits::Fetcher, Context};
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
	if auth.is_local() && query.fetch && !ctx.is_local(&aid) {
		let tx = ctx.db().begin().await?;
		let obj = ctx.fetch_activity(&aid, &tx).await?;
		tx.commit().await?;
		if obj.id != aid {
			return Err(crate::ApiError::Redirect(obj.id));
		}
	}

	let filter = Condition::all()
		.add(auth.filter_activities())
		.add(model::activity::Column::Id.eq(&aid));

	let activity = upub::Query::feed(auth.my_id(), true)
		.filter(filter)
		.into_model::<RichActivity>()
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?
		.load_batched_models(ctx.db())
		.await?;

	Ok(JsonLD(ctx.ap(activity).ld_context()))
}

