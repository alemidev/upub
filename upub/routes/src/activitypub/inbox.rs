use apb::{server::Inbox, Activity, ActivityType, Base};
use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{sea_query::IntoCondition, ColumnTrait};
use upub::{server::auth::{AuthIdentity, Identity}, Context};

use crate::builders::JsonLD;

use super::Pagination;


pub async fn get(
	State(ctx): State<Context>,
) -> upub::Result<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/inbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> upub::Result<JsonLD<serde_json::Value>> {
	crate::builders::paginate(
		upub::url!(ctx, "/inbox/page"),
		upub::model::addressing::Column::Actor.is_null()
			.into_condition(),
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
) -> upub::Result<()> {
	let Identity::Remote { domain: server, user: uid, .. } = auth else {
		if activity.activity_type() == Some(ActivityType::Delete) {
			// this is spammy af, ignore them!
			// we basically received a delete for a user we can't fetch and verify, meaning remote
			// deleted someone we never saw. technically we deleted nothing so we should return error,
			// but mastodon keeps hammering us trying to delete this user, so just make mastodon happy
			// and return 200 without even bothering checking this stuff
			// would be cool if mastodon played nicer with the network...
			return Ok(());
		}
		tracing::warn!("refusing unauthorized activity: {}", pretty_json!(activity));
		if matches!(auth, Identity::Anonymous) {
			return Err(upub::Error::unauthorized());
		} else {
			return Err(upub::Error::forbidden());
		}
	};

	let aid = activity.id().ok_or_else(|| upub::Error::field("id"))?.to_string();
	let actor = activity.actor().id().ok_or_else(|| upub::Error::field("actor"))?;

	if uid != actor {
		return Err(upub::Error::unauthorized());
	}

	tracing::debug!("processing federated activity: '{:#}'", activity);

	// TODO we could process Links and bare Objects maybe, but probably out of AP spec?
	match activity.activity_type().ok_or_else(upub::Error::bad_request)? {
		ActivityType::Activity => {
			tracing::warn!("skipping unprocessable base activity: {}", pretty_json!(activity));
			Err(StatusCode::UNPROCESSABLE_ENTITY.into()) // won't ingest useless stuff
		},

		// TODO emojireacts are NOT likes, but let's process them like ones for now maybe?
		ActivityType::Like | ActivityType::EmojiReact => Ok(ctx.like(server, activity).await?),
		ActivityType::Create => Ok(ctx.create(server, activity).await?),
		ActivityType::Follow => Ok(ctx.follow(server, activity).await?),
		ActivityType::Announce => Ok(ctx.announce(server, activity).await?),
		ActivityType::Accept(_) => Ok(ctx.accept(server, activity).await?),
		ActivityType::Reject(_) => Ok(ctx.reject(server, activity).await?),
		ActivityType::Undo => Ok(ctx.undo(server, activity).await?),
		ActivityType::Delete => Ok(ctx.delete(server, activity).await?),
		ActivityType::Update => Ok(ctx.update(server, activity).await?),

		_x => {
			tracing::info!("received unimplemented activity on inbox: {}", pretty_json!(activity));
			Err(StatusCode::NOT_IMPLEMENTED.into())
		},
	}
}
