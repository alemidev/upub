use apb::{Activity, ActivityType, Base};
use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{sea_query::IntoCondition, ActiveValue::{NotSet, Set}, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use upub::{model::job::JobType, selector::{RichActivity, RichFillable}, Context};

use crate::{AuthIdentity, Identity, builders::JsonLD};

use super::Pagination;


pub async fn get(
	State(ctx): State<Context>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(upub::url!(ctx, "/inbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let filter = upub::model::addressing::Column::Actor.is_null().into_condition();
	let (limit, offset) = page.pagination();
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
	crate::builders::collection_page(&upub::url!(ctx, "/inbox/page"), page, apb::Node::array(items))
}

pub async fn post(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>
) -> crate::ApiResult<StatusCode> {
	let Identity::Remote { domain: _server, user: uid, .. } = auth else {
		if matches!(activity.activity_type(), Ok(ActivityType::Delete)) {
			// this is spammy af, ignore them!
			// we basically received a delete for a user we can't fetch and verify, meaning remote
			// deleted someone we never saw. technically we deleted nothing so we should return error,
			// but mastodon keeps hammering us trying to delete this user, so just make mastodon happy
			// and return 200 without even bothering checking this stuff
			// would be cool if mastodon played nicer with the network...
			return Ok(StatusCode::OK);
		}
		tracing::warn!(
			"refusing unauthorized activity: {}",
			serde_json::to_string_pretty(&activity).expect("failed serializing to string serde_json::Value?")
		);
		if matches!(auth, Identity::Anonymous) {
			return Err(crate::ApiError::unauthorized());
		} else {
			return Err(crate::ApiError::forbidden());
		}
	};

	let aid = activity.id()?.to_string();
	let server = upub::Context::server(&aid);

	if activity.actor().id()? != uid {
		return Err(crate::ApiError::forbidden());
	}

	if let Some(_internal) = upub::model::activity::Entity::ap_to_internal(&aid, ctx.db()).await? {
		return Ok(StatusCode::OK); // already processed
	}

	let job = upub::model::job::ActiveModel {
		internal: NotSet,
		job_type: Set(JobType::Inbound),
		actor: Set(uid),
		target: Set(None),
		activity: Set(aid),
		payload: Set(Some(activity)),
		published: Set(chrono::Utc::now()),
		not_before: Set(chrono::Utc::now()),
		attempt: Set(0),
		error: Set(None),
	};

	upub::model::job::Entity::insert(job).exec(ctx.db()).await?;

	upub::downtime::unset(ctx.db(), &server).await?;

	Ok(StatusCode::ACCEPTED)
}
