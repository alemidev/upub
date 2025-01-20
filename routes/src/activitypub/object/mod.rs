pub mod replies;
pub mod context;
pub mod likes;
pub mod shares;

use apb::LD;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, QueryFilter, TransactionTrait};
use upub::{model, selector::{RichFillable, RichObject}, traits::Fetcher, Context};

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

	let filter = Condition::all()
		.add(auth.filter_objects())
		.add(model::object::Column::Id.eq(&oid));

	let object = upub::Query::feed(auth.my_id(), true)
		.filter(filter)
		.into_model::<RichObject>()
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?
		.load_batched_models(ctx.db())
		.await?;

	Ok(JsonLD(ctx.ap(object).ld_context()))
}
