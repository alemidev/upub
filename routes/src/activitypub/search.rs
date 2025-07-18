use apb::{BaseMut, CollectionMut, ObjectMut};
use axum::extract::{Query, State};
use sea_orm::{sea_query::{Expr, Alias}, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect, SelectColumns, TransactionTrait};
use upub::{selector::{RichFillable, RichObject}, traits::Fetcher, Context};

use crate::{builders::JsonLD, AuthIdentity};

use super::{PaginatedSearch, Pagination};


pub async fn objects(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<PaginatedSearch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	if !auth.is_local() && !ctx.cfg().security.allow_public_search {
		return Err(crate::ApiError::forbidden());
	}

	let is_exact_search = page.q.starts_with("https://") || page.q.starts_with("http://");

	if auth.is_local() && is_exact_search {
		let tx = ctx.db().begin().await?;
		ctx.fetch_object(&page.q, &tx).await?;
		tx.commit().await?;
	}

	// TODO search on URL is slow! maybe add index?
	let mut inner_filter = Condition::any()
		.add(upub::model::object::Column::Id.eq(&page.q))
		.add(upub::model::object::Column::Url.eq(&page.q));

	if !is_exact_search {
		inner_filter = inner_filter
			.add(upub::model::object::Column::Content.like(format!("%{}%", page.q)));
	}

	let filter = Condition::all()
		.add(auth.filter_objects())
		.add(inner_filter);

	// TODO lmao rethink this all
	//      still haven't redone this gg me
	//      have redone it but didnt rethink it properly so we're stuck with this bahahaha
	let p = Pagination {
		offset: page.offset,
		batch: page.batch,
		replies: Some(true),
	};

	let (limit, offset) = p.pagination();
	let items = upub::Query::objects(upub::query_feed_opts!(auth.my_id(), true))
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.order_by_desc(upub::model::object::Column::Published)
		.into_model::<RichObject>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/search/objects?q={}", page.q), p, apb::Node::array(items))
}

pub async fn actors(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(mut page): Query<PaginatedSearch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	if !auth.is_local() && !ctx.cfg().security.allow_public_search {
		return Err(crate::ApiError::forbidden());
	}

	let is_exact_search = page.q.starts_with('@') || page.q.starts_with("https://") || page.q.starts_with("http://");

	if auth.is_local() && is_exact_search {
		// allow searching with @user@domain format
		if page.q.starts_with('@') {
			if let Some((user, host)) = page.q.replacen('@', "", 1).split_once('@') {
				if let Some(webfinger) = ctx.webfinger(user, host).await? {
					page.q = webfinger;
				}
			}
		}
		ctx.fetch_user(&page.q, ctx.db()).await?;
	}

	let mut filter = Condition::any()
		.add(upub::model::actor::Column::Id.eq(&page.q));

	if !is_exact_search {
		filter = filter
			.add(upub::model::actor::Column::Name.like(format!("%{}%", page.q)))
			.add(upub::model::actor::Column::PreferredUsername.like(format!("%{}%", page.q)));
	};

	// TODO lmao rethink this all
	//      still haven't redone this gg me
	//      have redone it but didnt rethink it properly so we're stuck with this bahahaha
	let p = Pagination {
		offset: page.offset,
		batch: page.batch,
		replies: Some(true),
	};

	let (limit, offset) = p.pagination();
	let items = upub::model::actor::Entity::find()
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.order_by_desc(upub::model::actor::Column::Published)
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/search/actors?q={}", page.q), p, apb::Node::array(items))
}

pub async fn tags(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<PaginatedSearch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	if !auth.is_local() && !ctx.cfg().security.allow_public_search {
		return Err(crate::ApiError::forbidden());
	}

	// TODO lmao rethink this all
	//      still haven't redone this gg me
	//      have redone it but didnt rethink it properly so we're stuck with this bahahaha
	let p = Pagination {
		offset: page.offset,
		batch: page.batch,
		replies: Some(true),
	};

	let (limit, offset) = p.pagination();
	let items = upub::model::hashtag::Entity::find()
		.filter(upub::model::hashtag::Column::Name.like(format!("%{}%", page.q)))
		.select_only()
		.select_column(upub::model::hashtag::Column::Name)
		.column_as(upub::model::hashtag::Column::Name.count(), "count")
		.group_by(upub::model::hashtag::Column::Name)
		.order_by(Expr::col(Alias::new("count")), sea_orm::Order::Desc)
		.limit(limit)
		.offset(offset)
		.into_tuple::<(String, i64)>()
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|(name, count)| apb::new()
			.set_collection_type(Some(apb::CollectionType::Collection))
			.set_id(Some(upub::url!(ctx, "/tags/{name}")))
			.set_name(Some(name))
			.set_total_items(Some(count.max(0) as u64))
		)
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/search/tags?q={}", page.q), p, apb::Node::array(items))
}
