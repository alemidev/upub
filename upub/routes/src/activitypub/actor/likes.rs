use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait};

use upub::{model, selector::{RichObject, BatchFillable}, Context};

use crate::{activitypub::Pagination, builders::JsonLD, ApiError, AuthIdentity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(upub::url!(ctx, "/actors/{id}/liked"), None)
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

	if !auth.is(&uid) && !config.map_or(false, |x| x.show_liked_objects) {
		return Err(ApiError::forbidden());
	}

	let (limit, offset) = page.pagination();

	let mut select = upub::Query::objects(auth.my_id())
		.join(sea_orm::JoinType::InnerJoin, upub::model::object::Relation::Likes.def())
		.filter(auth.filter_objects())
		.filter(upub::model::like::Column::Actor.eq(user.internal))
		.order_by_desc(upub::model::like::Column::Published)
		.limit(limit)
		.offset(offset);

	let items : Vec<serde_json::Value> = select
		.into_model::<RichObject>()
		.all(ctx.db())
		.await?
		.with_batched::<upub::model::attachment::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::mention::Entity>(ctx.db())
		.await?
		.with_batched::<upub::model::hashtag::Entity>(ctx.db())
		.await?
		.into_iter()
		.map(|x| ctx.ap(x))
		.collect();
	
	crate::builders::paginate_feed(
		upub::url!(ctx, "/actors/{id}/outbox/page"),
		auth.filter_objects(),
		&ctx,
		page,
		auth.my_id(),
		false,
	)
		.await
}
