use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ActiveValue::{NotSet, Set}, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use upub::{model, selector::{RichActivity, RichFillable}, traits::Fetcher, Context};

use crate::{activitypub::{CreationResult, Pagination, TryFetch}, builders::JsonLD, AuthIdentity, Identity};

pub async fn get(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(id): Path<String>,
	Query(query): Query<TryFetch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(&id);
	if auth.is_local() && query.fetch && !ctx.is_local(&uid) {
		ctx.fetch_outbox(&uid, ctx.db()).await?;
	}

	crate::builders::collection(upub::url!(ctx, "/actors/{id}/outbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(&id);
	let filter = Condition::all()
		.add(auth.filter_activities())
		.add(
			Condition::any()
				.add(model::activity::Column::Actor.eq(&uid))
				.add(model::object::Column::AttributedTo.eq(&uid))
				.add(model::object::Column::Audience.eq(&uid))
		);

	let (limit, offset) = page.pagination();
	// by default we want replies because servers don't know about our api and need to see everything
	let items = upub::Query::feed(auth.my_id(), page.replies.unwrap_or(true))
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

	crate::builders::collection_page(&upub::url!(ctx, "/actors/{id}/outbox/page"), page, apb::Node::array(items))
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
				payload: Set(Some(activity)),
				error: Set(None),
			};

			model::job::Entity::insert(job).exec(ctx.db()).await?;

			ctx.wake_workers(); // process immediately

			Ok(CreationResult(aid))
		}
	}
}
