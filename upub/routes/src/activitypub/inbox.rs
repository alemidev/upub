use apb::{Activity, ActivityType, Base};
use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{ActiveValue::{NotSet, Set}, EntityTrait};
use upub::{model::job::JobType, Context};

use crate::{AuthIdentity, Identity, builders::JsonLD};

use super::Pagination;


pub async fn get(
	State(ctx): State<Context>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/inbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	crate::builders::paginate_activities(
		upub::url!(ctx, "/inbox/page"),
		auth.filter_activities(),
		ctx.db(),
		page,
		auth.my_id(),
		false,
	)
		.await
}

macro_rules! pretty_json {
	($json:ident) => {
		serde_json::to_string_pretty(&$json).expect("failed serializing to string serde_json::Value")
	}
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
		tracing::warn!("refusing unauthorized activity: {}", pretty_json!(activity));
		if matches!(auth, Identity::Anonymous) {
			return Err(crate::ApiError::unauthorized());
		} else {
			return Err(crate::ApiError::forbidden());
		}
	};

	let aid = activity.id()?.to_string();

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
		attempt: Set(0)
	};

	upub::model::job::Entity::insert(job).exec(ctx.db()).await?;

	Ok(StatusCode::ACCEPTED)
}
