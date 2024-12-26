use apb::{BaseMut, CollectionMut, CollectionPageMut, LD};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, ConnectionTrait, PaginatorTrait, QueryFilter, QuerySelect, SelectColumns};
use upub::{model, traits::Fetcher, Context};

use crate::{activitypub::{Pagination, TryFetch}, builders::JsonLD, ApiResult, AuthIdentity, Identity};

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

	let replies_count = total_replies(&oid, &auth, ctx.db()).await?;
	let replies_ids = replies_ids(&oid, &auth, ctx.db(), 20, 0).await?;

	let first = apb::new()
		.set_id(Some(upub::url!(ctx, "/objects/{id}/replies/page")))
		.set_collection_type(Some(apb::CollectionType::OrderedCollectionPage))
		.set_next(apb::Node::link(upub::url!(ctx, "/objects/{id}/replies/page?offset=20")))
		.set_ordered_items(apb::Node::links(replies_ids));

	Ok(JsonLD(
		apb::new()
			.set_id(Some(upub::url!(ctx, "/objects/{id}/replies")))
			.set_collection_type(Some(apb::CollectionType::Collection))
			.set_total_items(Some(replies_count))
			.set_first(apb::Node::object(first))
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

	let replies_ids = replies_ids(&oid, &auth, ctx.db(), limit, offset).await?;

	crate::builders::collection_page(
		&page_id,
		offset,
		limit,
		apb::Node::links(replies_ids)
	)
}

async fn replies_ids(oid: &str, auth: &Identity, db: &impl ConnectionTrait, limit: u64, offset: u64) -> ApiResult<Vec<String>> {
	let res = upub::Query::objects(auth.my_id())
		.limit(limit)
		.offset(offset)
		.filter(auth.filter_objects())
		.filter(model::object::Column::InReplyTo.eq(oid))
		.select_only()
		.select_column(model::object::Column::Id)
		.into_tuple::<String>()
		.all(db)
		.await?;
	Ok(res)
}

async fn total_replies(oid: &str, auth: &Identity, db: &impl ConnectionTrait) -> ApiResult<u64> {
	let count = upub::Query::objects(None)
		.filter(auth.filter_objects())
		.filter(model::object::Column::InReplyTo.eq(oid))
		.count(db)
		.await?;
	Ok(count)
}
