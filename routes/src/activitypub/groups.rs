use axum::extract::{Query, State};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, PaginatorTrait};

use upub::{model, Context};

use crate::{activitypub::Pagination, builders::JsonLD};

pub async fn get(
	State(ctx): State<Context>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let count = model::actor::Entity::find()
		.filter(model::actor::Column::ActorType.eq(apb::ActorType::Group))
		.count(ctx.db())
		.await?;
	crate::builders::collection(upub::url!(ctx, "/groups"), Some(count as u64))
}

pub async fn page(
	State(ctx): State<Context>,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let (limit, offset) = page.pagination();

	let groups = model::actor::Entity::find()
		.filter(model::actor::Column::ActorType.eq(apb::ActorType::Group))
		.limit(limit)
		.offset(offset)
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|x| ctx.ap(x))
		.collect();


	crate::builders::collection_page(
		&upub::url!(ctx, "/groups/page"),
		page,
		apb::Node::array(groups),
	)
}
