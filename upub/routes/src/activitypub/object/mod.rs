pub mod replies;
pub mod context;

use apb::LD;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter, TransactionTrait};
use upub::{model, selector::{BatchFillable, RichObject}, traits::Fetcher, Context};

use crate::{builders::JsonLD, AuthIdentity};

use super::TryFetch;

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<TryFetch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let oid = ctx.oid(&id);
	if auth.is_local() && query.fetch && !ctx.is_local(&oid) {
		let tx = ctx.db().begin().await?;
		let obj = ctx.fetch_object(&oid, &tx).await?;
		tx.commit().await?;
		// some implementations serve statuses on different urls than their AP id
		if obj.id != oid {
			return Err(crate::ApiError::Redirect(upub::url!(ctx, "/objects/{}", ctx.id(&obj.id))));
		}
	}

	let item = upub::Query::objects(auth.my_id())
		.filter(auth.filter_objects())
		.filter(model::object::Column::Id.eq(&oid))
		.into_model::<RichObject>()
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?
		.with_batched::<upub::model::attachment::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::mention::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::hashtag::Entity>(ctx.db())
		.await?;

	Ok(JsonLD(ctx.ap(item).ld_context()))
}
