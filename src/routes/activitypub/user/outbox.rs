use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, QueryFilter, QuerySelect};

use apb::{server::Outbox, AcceptType, ActivityType, Base, BaseType, ObjectType, RejectType};
use crate::{errors::UpubError, model::{self, addressing::EmbeddedActivity}, routes::activitypub::{jsonld::LD, CreationResult, JsonLD, Pagination}, server::{auth::{AuthIdentity, Identity}, Context}, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	Ok(JsonLD(
		ctx.ap_collection(&url!(ctx, "/users/{id}/outbox"), None).ld_context()
	))
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let uid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.uid(id.clone())
	};
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let activities = model::addressing::Entity::find_activities()
		.filter(model::activity::Column::Actor.eq(&uid))
		.filter(auth.filter_condition())
		.limit(limit)
		.offset(offset)
		.into_model::<EmbeddedActivity>()
		.all(ctx.db()).await?;

	let mut out = Vec::new();
	for activity in activities {
		out.push(activity.ap_filled(ctx.db()).await?);
	}

	Ok(JsonLD(
		ctx.ap_collection_page(
			&url!(ctx, "/users/{id}/outbox/page"),
			offset, limit, out,
		).ld_context()
	))
}

pub async fn post(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>,
) -> Result<CreationResult, UpubError> {
	match auth {
		Identity::Anonymous => Err(StatusCode::UNAUTHORIZED.into()),
		Identity::Remote(_) => Err(StatusCode::NOT_IMPLEMENTED.into()),
		Identity::Local(uid) => if ctx.uid(id.clone()) == uid {
			match activity.base_type() {
				None => Err(StatusCode::BAD_REQUEST.into()),

				Some(BaseType::Link(_)) => Err(StatusCode::UNPROCESSABLE_ENTITY.into()),

				Some(BaseType::Object(ObjectType::Note)) =>
					Ok(CreationResult(ctx.create_note(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) =>
					Ok(CreationResult(ctx.create(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) =>
					Ok(CreationResult(ctx.like(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) =>
					Ok(CreationResult(ctx.follow(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Undo))) =>
					Ok(CreationResult(ctx.undo(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Accept(AcceptType::Accept)))) =>
					Ok(CreationResult(ctx.accept(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Reject(RejectType::Reject)))) =>
					Ok(CreationResult(ctx.reject(uid, activity).await?)),

				Some(_) => Err(StatusCode::NOT_IMPLEMENTED.into()),
			}
		} else {
			Err(StatusCode::FORBIDDEN.into())
		}
	}
}
