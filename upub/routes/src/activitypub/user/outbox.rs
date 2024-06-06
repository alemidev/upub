use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ActiveValue::{NotSet, Set}, ColumnTrait, Condition, EntityTrait};

use upub::{model, Context};

use crate::{activitypub::{CreationResult, Pagination}, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/actors/{id}/outbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(&id);
	crate::builders::paginate(
		upub::url!(ctx, "/actors/{id}/outbox/page"),
		Condition::all()
			.add(auth.filter_condition())
			.add(
				Condition::any()
					.add(model::activity::Column::Actor.eq(&uid))
					.add(model::object::Column::AttributedTo.eq(&uid))
				),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}

pub async fn post(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>,
) -> crate::ApiResult<CreationResult> {
	match auth {
		Identity::Anonymous => Err(StatusCode::UNAUTHORIZED.into()),
		Identity::Remote { .. } => Err(StatusCode::NOT_IMPLEMENTED.into()),
		Identity::Local { id: uid, .. } => {
			if ctx.uid(&id) != uid {
				return Err(crate::ApiError::forbidden());
			}

			tracing::debug!("enqueuing new local activity: {}", serde_json::to_string(&activity).unwrap_or_default());
			let aid = ctx.aid(&Context::new_id());

			let job = model::job::ActiveModel {
				internal: NotSet,
				activity: Set(aid.clone()),
				job_type: Set(model::job::JobType::Outbound),
				actor: Set(uid.clone()),
				target: Set(None),
				published: Set(chrono::Utc::now()),
				not_before: Set(chrono::Utc::now()),
				attempt: Set(0),
				payload: Set(Some(serde_json::to_string(&activity).expect("failed serializing back json object"))),
			};

			model::job::Entity::insert(job).exec(ctx.db()).await?;

			Ok(CreationResult(aid))
		}
	}
}
