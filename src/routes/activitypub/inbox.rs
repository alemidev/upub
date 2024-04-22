use apb::{server::Inbox, Activity, ActivityType};
use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{QueryFilter, QuerySelect};

use crate::{errors::UpubError, model::{self, addressing::EmbeddedActivity}, server::{auth::{AuthIdentity, Identity}, Context}, url};

use super::{jsonld::LD, JsonLD, Pagination};


pub async fn get(
	State(ctx): State<Context>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	Ok(JsonLD(ctx.ap_collection(&url!(ctx, "/inbox"), None).ld_context()))
}

pub async fn page(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	let activities = model::addressing::Entity::find_activities()
		.filter(auth.filter_condition())
		.limit(limit)
		.offset(offset)
		.into_model::<EmbeddedActivity>()
		.all(ctx.db())
		.await?;
	let mut out = Vec::new();
	for activity in activities {
		out.push(activity.ap_filled(ctx.db()).await?);
	}
	Ok(JsonLD(
		ctx.ap_collection_page(
			&url!(ctx, "/inbox/page"),
			offset, limit, out,
		).ld_context()
	))
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
) -> crate::Result<()> {
	let Identity::Remote(server) = auth else {
		if activity.activity_type() != Some(ActivityType::Delete) { // this is spammy af, ignore them!
			tracing::warn!("refusing unauthorized activity: {}", pretty_json!(activity));
		}
		if matches!(auth, Identity::Anonymous) {
			return Err(UpubError::unauthorized());
		} else {
			return Err(UpubError::forbidden());
		}
	};

	let Some(actor) = activity.actor().id() else {
		return Err(UpubError::bad_request());
	};

	// TODO add whitelist of relays
	if !server.ends_with(&Context::server(&actor)) {
		return Err(UpubError::unauthorized());
	}

	// TODO we could process Links and bare Objects maybe, but probably out of AP spec?
	match activity.activity_type().ok_or_else(UpubError::bad_request)? {
		ActivityType::Activity => {
			tracing::warn!("skipping unprocessable base activity: {}", pretty_json!(activity));
			Err(StatusCode::UNPROCESSABLE_ENTITY.into()) // won't ingest useless stuff
		},

		ActivityType::Create => Ok(ctx.create(server, activity).await?),
		ActivityType::Like => Ok(ctx.like(server, activity).await?),
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
