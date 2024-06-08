use axum::extract::{Path, Query, State};
use sea_orm::{sea_query::IntoCondition, ColumnTrait};

use upub::Context;

use crate::{activitypub::Pagination, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	match auth {
		Identity::Anonymous => Err(crate::ApiError::forbidden()),
		Identity::Remote { .. } => Err(crate::ApiError::forbidden()),
		Identity::Local { id: user, .. } => if ctx.uid(&id) == user {
			crate::builders::collection(&upub::url!(ctx, "/actors/{id}/feed"), None)
		} else {
			Err(crate::ApiError::forbidden())
		},
	}
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let Identity::Local { id: uid, internal } = &auth else {
		// local inbox is only for local users
		return Err(crate::ApiError::forbidden());
	};
	if uid != &ctx.uid(&id) {
		return Err(crate::ApiError::forbidden());
	}

	crate::builders::paginate_activities(
		upub::url!(ctx, "/actors/{id}/feed/page"),
		upub::model::addressing::Column::Actor.eq(*internal).into_condition(),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}
