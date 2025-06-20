use apb::LD;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, EntityTrait, Iterable, QueryFilter, QueryOrder, QuerySelect, RelationTrait, SelectColumns, EntityName, Iden};

use upub::{model, selector::{RichActivity, RichFillable, RichObjectOrActor}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let lid = ctx.lid(&id);

	let list = list_if_authorized(&ctx, &lid, &auth).await?;

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

	let list = list_if_authorized(&ctx, &lid, &auth).await?;

	let mut select = model::list_element::Entity::find()
		.join(sea_orm::JoinType::LeftJoin, model::list_element::Relation::Actors.def())
		.join(sea_orm::JoinType::LeftJoin, model::list_element::Relation::Objects.def())
		.filter(model::list_element::Column::List.eq(list.internal))
		.limit(limit)
		.offset(offset);


	// TODO this should be in a Query::__ helper maybe?
	for col in model::actor::Column::iter() {
		select = select.select_column_as(col, format!("{}{}", model::actor::Entity.table_name(), col.to_string()));
	}

	for col in model::object::Column::iter() {
		select = select.select_column_as(col, format!("{}{}", model::object::Entity.table_name(), col.to_string()));
	}

	let list_items = select
		.into_model::<RichObjectOrActor>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|x| ctx.ap(x))
		.collect();


	crate::builders::collection_page(
		&upub::url!(ctx, "{lid}/page"),
		page,
		apb::Node::array(list_items),
	)
}

pub async fn feed(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let lid = ctx.lid(&id);

	list_if_authorized(&ctx, &lid, &auth).await?;

	crate::builders::collection(format!("{lid}/feed"), None)
}

pub async fn feed_page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let lid = ctx.lid(&id);
	let (limit, offset) = page.pagination();

	let list = list_if_authorized(&ctx, &lid, &auth).await?;

	let actors = model::list_element::Entity::find()
		.filter(model::list_element::Column::Actor.is_not_null())
		.filter(model::list_element::Column::List.eq(list.internal))
		.join(sea_orm::JoinType::InnerJoin, model::list_element::Relation::Actors.def())
		.select_only()
		.select_column(model::actor::Column::Id)
		.into_tuple::<String>()
		.all(ctx.db())
		.await?;

	let filter = Condition::all()
		.add(auth.filter_activities())
		.add(upub::model::activity::Column::Actor.is_in(actors));
	// TODO this IN clause above may be taxing for large lists becase we send in a lot of long
	//      strings to compare, maybe it's cheaper to join on Actors too and compare by internal ids?

	let feed = upub::Query::feed(upub::query_feed_opts!(auth.my_id(), page.replies()))
		.filter(filter)
		.limit(limit)
		.offset(offset)
		.order_by_desc(upub::model::addressing::Column::Published)
		.order_by_desc(upub::model::activity::Column::Internal)
		.into_model::<RichActivity>()
		.all(ctx.db())
		.await?
		.load_batched_models(ctx.db())
		.await?
		.into_iter()
		.map(|item| ctx.ap(item))
		.collect();

	crate::builders::collection_page(
		&format!("{lid}/feed/page"),
		page,
		apb::Node::array(feed),
	)
}





async fn list_if_authorized(ctx: &Context, lid: &str, auth: &Identity) -> crate::ApiResult<model::list::Model> {
	let list = model::list::Entity::find_by_ap_id(lid)
		.one(ctx.db())
		.await?
		.ok_or(sea_orm::DbErr::RecordNotFound(lid.to_string()))?;

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

	Ok(list)
}
