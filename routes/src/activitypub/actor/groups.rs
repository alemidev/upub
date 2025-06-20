use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, SelectColumns};

use upub::{model, Context};

use crate::{activitypub::Pagination, builders::JsonLD, ApiError, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(&id);

	if !auth.is(&uid) {
		return Err(crate::ApiError::unauthorized());
	}

	let user = model::actor::Entity::find_by_ap_id(&uid)
		.one(ctx.db())
		.await?
		.ok_or_else(ApiError::not_found)?;

	let filter = Condition::all()
		.add(model::relation::Column::Accept.is_not_null())
		.add(upub::model::relation::Column::Follower.eq(user.internal))
		.add(model::actor::Column::ActorType.eq(apb::ActorType::Group));

	let join = model::relation::Relation::ActorsFollowing.def();

	let groups_count = model::relation::Entity::find()
		.filter(filter)
		.join(sea_orm::JoinType::LeftJoin, join)
		.select_only()
		.select_column(model::actor::Column::Id)
		.count(ctx.db())
		.await?;

	crate::builders::collection(upub::url!(ctx, "/actors/{id}/groups"), Some(groups_count))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let (limit, _offset) = page.pagination();
	let uid = ctx.uid(&id);

	if !auth.is(&uid) {
		return Err(crate::ApiError::unauthorized());
	}

	let user = model::actor::Entity::find_by_ap_id(&uid)
		.one(ctx.db())
		.await?
		.ok_or_else(ApiError::not_found)?;

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
