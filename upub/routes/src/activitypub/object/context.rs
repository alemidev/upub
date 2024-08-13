use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, PaginatorTrait, QueryFilter, QuerySelect};
use upub::{model, selector::{BatchFillable, RichActivity}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	let count = upub::Query::feed(auth.my_id(), true)
		.filter(auth.filter())
		.filter(model::object::Column::Context.eq(&context))
		.count(ctx.db())
		.await?;

	crate::builders::collection(&upub::url!(ctx, "/objects/{id}/context"), Some(count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let context = ctx.oid(&id);

	let mut filter = Condition::any()
		.add(auth.filter());

	if let Identity::Local { ref id, .. } = auth {
		filter = filter.add(model::object::Column::AttributedTo.eq(id));
	}

	filter = Condition::all()
		.add(model::object::Column::Context.eq(context))
		.add(filter);

	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);


	let items = upub::Query::feed(auth.my_id(), true)
		.filter(filter)
		// TODO also limit to only local activities
		.limit(limit)
		.offset(offset)
		.into_model::<RichActivity>()
		.all(ctx.db())
		.await?
		.with_batched::<upub::model::attachment::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::mention::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::hashtag::Entity>(ctx.db())
		.await?;

	let items : Vec<serde_json::Value> = items
		.into_iter()
		.map(|item| item.ap())
		.collect();

	crate::builders::collection_page(&id, offset, limit, items)
}
