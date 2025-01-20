use apb::{BaseMut, CollectionMut, LD};
use axum::extract::{Path, Query, State};
use sea_orm::{sea_query::IntoColumnRef, ColumnTrait, EntityName, EntityTrait, Iden, Iterable, QueryFilter, QueryOrder, QuerySelect, RelationTrait, SelectColumns};
use upub::{selector::{RichActivity, RichFillable}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let oid = ctx.oid(&id);

	let object = upub::model::object::Entity::find_by_ap_id(&oid)
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?;

	Ok(JsonLD(
		apb::new()
			.set_id(Some(upub::url!(ctx, "/objects/{id}/shares")))
			.set_collection_type(Some(apb::CollectionType::Collection))
			.set_total_items(Some(object.announces as u64))
			.set_first(apb::Node::link(upub::url!(ctx, "/objects/{id}/shares/page")))
			.ld_context()
	))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let oid = ctx.oid(&id);
	let internal = upub::model::object::Entity::ap_to_internal(&oid, ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?;

	let (limit, offset) = page.pagination();
	let mut select = upub::model::announce::Entity::find()
		.distinct_on([
			(upub::model::announce::Entity, upub::model::announce::Column::Published).into_column_ref(),
			(upub::model::activity::Entity, upub::model::activity::Column::Internal).into_column_ref(),
		])
		.join(sea_orm::JoinType::InnerJoin, upub::model::announce::Relation::Activities.def())
		.join(sea_orm::JoinType::InnerJoin, upub::model::activity::Relation::Addressing.def())
		.join(sea_orm::JoinType::InnerJoin, upub::model::activity::Relation::Objects.def())
		.filter(auth.filter_activities())
		.filter(upub::model::announce::Column::Object.eq(internal))
		.order_by_desc(upub::model::announce::Column::Published)
		.order_by_desc(upub::model::activity::Column::Internal)
		.limit(limit)
		.offset(offset)
		.select_only();

	for col in upub::model::activity::Column::iter() {
		select = select.select_column_as(col, format!("{}{}", upub::model::activity::Entity.table_name(), col.to_string()));
	}

	select = select.select_column_as(
		upub::model::addressing::Column::Published,
		format!("{}{}", upub::model::addressing::Entity.table_name(), upub::model::addressing::Column::Published.to_string())
	);

	let items = select
		.into_model::<RichActivity>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/objects/{id}/shares/page"), page, apb::Node::array(items))
}
