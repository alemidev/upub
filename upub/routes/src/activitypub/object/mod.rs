pub mod replies;
pub mod context;

use apb::{BaseMut, CollectionMut, ObjectMut, LD};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter, QuerySelect, SelectColumns, TransactionTrait};
use upub::{model, selector::{BatchFillable, RichActivity}, traits::Fetcher, Context};

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
		.filter(auth.filter())
		.filter(model::object::Column::Id.eq(&oid))
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

	let mut replies = apb::Node::Empty;
	
	if ctx.cfg().security.show_reply_ids {
		let replies_ids = upub::Query::objects(auth.my_id())
			.filter(auth.filter())
			.filter(model::object::Column::InReplyTo.eq(oid))
			.select_only()
			.select_column(model::object::Column::Id)
			.into_tuple::<String>()
			.all(ctx.db())
			.await?;

		replies = apb::Node::object(
			apb::new()
				.set_id(Some(&upub::url!(ctx, "/objects/{id}/replies")))
				.set_first(apb::Node::link(upub::url!(ctx, "/objects/{id}/replies/page")))
				.set_collection_type(Some(apb::CollectionType::Collection))
				.set_total_items(item.object.as_ref().map(|x| x.replies as u64))
				.set_items(apb::Node::links(replies_ids))
		);
	}
	
	Ok(JsonLD(
		item.object_ap()
			.set_replies(replies)
			.ld_context()
	))
}
