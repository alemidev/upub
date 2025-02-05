use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, Iterable, QueryFilter, QueryOrder, QuerySelect, RelationTrait, SelectColumns, Iden, EntityName};

use upub::{model, selector::{RichFillable, RichObject}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, ApiError, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(upub::url!(ctx, "/actors/{id}/likes"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(&id);
	let (user, config) = model::actor::Entity::find_by_ap_id(&uid)
		.find_also_related(model::config::Entity)
		.one(ctx.db())
		.await?
		.ok_or_else(ApiError::not_found)?;

	if !auth.is(&uid) && !config.map_or(true, |x| x.show_liked_objects) {
		return Err(ApiError::forbidden());
	}

	let (limit, offset) = page.pagination();

	let mut select = upub::model::like::Entity::find()
		.distinct()
		.join(sea_orm::JoinType::InnerJoin, upub::model::like::Relation::Objects.def())
		.join(sea_orm::JoinType::InnerJoin, upub::model::like::Relation::Activities.def())
		.join(sea_orm::JoinType::InnerJoin, upub::model::activity::Relation::Addressing.def())
		.filter(auth.filter_activities())
		.filter(upub::model::like::Column::Actor.eq(user.internal))
		.order_by_desc(upub::model::like::Column::Published)
		.select_only()
		.select_column(upub::model::like::Column::Published);

	for col in upub::model::object::Column::iter() {
		select = select.select_column_as(col, format!("{}{}", model::object::Entity.table_name(), col.to_string()));
	}

	let items : Vec<serde_json::Value> = select
		.limit(limit)
		.offset(offset)
		.into_model::<RichObject>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|x| ctx.ap(x))
		.collect();

	crate::builders::collection_page(&upub::url!(ctx, "/actors/{id}/likes/page"), page, apb::Node::array(items))
}
