use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, SelectColumns, RelationTrait};

use upub::{model, Context};

use crate::{activitypub::Pagination, builders::JsonLD, ApiError, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(upub::url!(ctx, "/actors/{id}/groups"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let (limit, _offset) = page.pagination();

	let user = model::actor::Entity::find_by_ap_id(&ctx.uid(&id))
		.one(ctx.db())
		.await?
		.ok_or_else(ApiError::not_found)?;

	if auth.my_id().is_none_or(|x| x != user.internal) {
		return Err(crate::ApiError::unauthorized());
	}

	let filter = Condition::all()
		.add(model::relation::Column::Accept.is_not_null())
		.add(upub::model::relation::Column::Follower.eq(user.internal))
		.add(model::actor::Column::ActorType.eq(apb::ActorType::Group));

	let join = model::relation::Relation::ActorsFollowing.def();

	let following = model::relation::Entity::find()
		.filter(filter)
		.join(sea_orm::JoinType::LeftJoin, join)
		.select_only()
		.select_column(model::actor::Column::Id)
		.limit(limit)
		.offset(page.offset.unwrap_or(0))
		.into_tuple::<String>()
		.all(ctx.db())
		.await?;

	crate::builders::collection_page(
		&upub::url!(ctx, "/actors/{id}/groups/page"),
		page,
		apb::Node::links(following),
	)
}
