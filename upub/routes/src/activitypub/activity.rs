use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter, TransactionTrait};
use upub::{model, selector::{BatchFillable, RichActivity}, traits::Fetcher, Context};
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

	let row = upub::Query::feed(auth.my_id())
		.filter(auth.filter())
		.filter(model::activity::Column::Id.eq(&aid))
		.into_model::<RichActivity>()
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?
		.with_batched::<upub::model::attachment::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::mention::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::hashtag::Entity>(ctx.db())
		.await?;

	Ok(JsonLD(row.ap().ld_context()))
}

