use apb::{BaseMut, CollectionMut, LD};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use upub::{model, selector::RichObject, traits::Fetcher, Context};

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

	let total_replies = upub::Query::objects(None)
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
	let page_id = upub::url!(ctx, "/objects/{id}/replies/page");
	let oid = ctx.oid(&id);
	let (limit, offset) = page.pagination();

	// TODO kinda weird ignoring this but its weirder to exclude replies from replies view...
	page.replies = Some(true);

	let res = upub::Query::objects(auth.my_id())
		.limit(limit)
		.offset(offset)
		.filter(auth.filter_objects())
		.filter(model::object::Column::InReplyTo.eq(oid))
		.order_by_desc(model::object::Column::Published)
		.into_model::<RichObject>()
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|x| ctx.ap(x))
		.collect();

	crate::builders::collection_page(
		&page_id,
		offset,
		limit,
		apb::Node::array(res)
	)
}
