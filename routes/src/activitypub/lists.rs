use apb::LD;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, SelectColumns, RelationTrait};

use upub::{model, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let lid = ctx.lid(&id);

	let list = model::list::Entity::find_by_ap_id(&lid)
		.one(ctx.db())
		.await?
		.ok_or(sea_orm::DbErr::RecordNotFound(lid))?;

	if let (_, Some(config)) = model::actor::Entity::find_by_ap_id(&list.attributed_to)
		.find_also_related(model::config::Entity)
		.one(ctx.db())
		.await?
		.ok_or(sea_orm::DbErr::RecordNotFound(list.attributed_to.clone()))?
	{
		if !config.show_lists && !auth.is(&list.attributed_to) {
			return Err(crate::ApiError::unauthorized());
		}
	}

	Ok(JsonLD(ctx.ap(list).ld_context()))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let lid = ctx.lid(&id);
	let (limit, offset) = page.pagination();

	let list = model::list::Entity::find_by_ap_id(&lid)
		.one(ctx.db())
		.await?
		.ok_or(sea_orm::DbErr::RecordNotFound(lid))?;

	if let (_, Some(config)) = model::actor::Entity::find_by_ap_id(&list.attributed_to)
		.find_also_related(model::config::Entity)
		.one(ctx.db())
		.await?
		.ok_or(sea_orm::DbErr::RecordNotFound(list.attributed_to.clone()))?
	{
		if !config.show_lists && !auth.is(&list.attributed_to) {
			return Err(crate::ApiError::unauthorized());
		}
	}

	let list_items = model::list_element::Entity::find()
		.join(sea_orm::JoinType::InnerJoin, model::list_element::Relation::Actors.def())
		.join(sea_orm::JoinType::InnerJoin, model::list_element::Relation::Objects.def())
		.filter(model::list_element::Column::List.eq(list.internal))
		.select_only()
		.select_column(model::actor::Column::Id)
		.select_column(model::object::Column::Id)
		.limit(limit)
		.offset(offset)
		.into_tuple::<(Option<String>, Option<String>)>()
		.all(ctx.db())
		.await?
		.into_iter()
		.filter_map(|(actor, object)| match (actor, object) {
			(Some(_a), Some(_o)) => None, // this should never happen?
			(Some(a), None) => Some(a),
			(None, Some(o)) => Some(o),
			(None, None) => None,
		})
		.collect::<Vec<String>>();

	crate::builders::collection_page(
		&upub::url!(ctx, "/lists/{id}/page"),
		page,
		apb::Node::links(list_items),
	)
}
	
