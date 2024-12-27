use apb::{BaseMut, CollectionMut, LD};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use upub::{model, selector::{RichFillable, RichObject}, traits::Fetcher, Context};

use crate::{activitypub::{Pagination, TryFetch}, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(q): Query<TryFetch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let oid = ctx.oid(&id);
	if auth.is_local() && q.fetch {
		// TODO a task should do this, not the web handler!
		//      so we dont keep clients waiting and we limit
		//      concurrent possible crawlers
		//      however the results given immediately would
		//      become inaccurate!!
		ctx.fetch_thread(&oid, ctx.db()).await?;
	}

	let total_replies = upub::Query::objects(None, true)
		.filter(auth.filter_objects())
		.filter(model::object::Column::InReplyTo.eq(&oid))
		.count(ctx.db())
		.await?;

	Ok(JsonLD(
		apb::new()
			.set_id(Some(upub::url!(ctx, "/objects/{id}/replies")))
			.set_collection_type(Some(apb::CollectionType::Collection))
			.set_total_items(Some(total_replies))
			.set_first(apb::Node::link(upub::url!(ctx, "/objects/{id}/replies/page")))
			.ld_context()
	))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(mut page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let oid = ctx.oid(&id);

	// TODO kinda weird ignoring this but its weirder to exclude replies from replies view...
	page.replies = Some(true);

	let filter = Condition::all()
		.add(auth.filter_objects())
		.add(model::object::Column::InReplyTo.eq(oid));

	let (limit, offset) = page.pagination();
	let items = upub::Query::feed(auth.my_id(), page.replies.unwrap_or(true))
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.order_by_desc(upub::model::addressing::Column::Published)
		.order_by_desc(upub::model::activity::Column::Internal)
		.into_model::<RichObject>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/objects/{id}/replies/page"), page, apb::Node::array(items))
}
