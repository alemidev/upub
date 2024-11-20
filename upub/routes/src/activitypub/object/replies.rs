use apb::{BaseMut, CollectionMut, LD};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, QueryFilter, QuerySelect, SelectColumns};
use upub::{model, traits::Fetcher, Context};

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

	let replies_ids = upub::Query::objects(auth.my_id())
		.filter(auth.filter())
		.filter(model::object::Column::InReplyTo.eq(ctx.oid(&id)))
		.select_only()
		.select_column(model::object::Column::Id)
		.into_tuple::<String>()
		.all(ctx.db())
		.await?;

	Ok(JsonLD(
		apb::new()
			.set_id(Some(upub::url!(ctx, "/objects/{id}/replies")))
			.set_collection_type(Some(apb::CollectionType::Collection))
			.set_first(apb::Node::link(upub::url!(ctx, "/objects/{id}/replies/page")))
			.set_total_items(Some(replies_ids.len() as u64))
			.set_items(apb::Node::links(replies_ids))
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

	// TODO kinda weird ignoring this but its weirder to exclude replies from replies view...
	page.replies = Some(true);

	crate::builders::paginate_feed(
		page_id,
		Condition::all()
			.add(auth.filter())
			.add(model::object::Column::InReplyTo.eq(oid)),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}
