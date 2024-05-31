use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition};

use apb::{server::Outbox, AcceptType, ActivityType, Base, BaseType, ObjectType, RejectType};
use upub::{model, server::auth::{AuthIdentity, Identity}, Context};

use crate::{activitypub::{CreationResult, Pagination}, builders::JsonLD};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> upub::Result<JsonLD<serde_json::Value>> {
	crate::builders::collection(&upub::url!(ctx, "/actors/{id}/outbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> upub::Result<JsonLD<serde_json::Value>> {
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
) -> upub::Result<CreationResult> {
	match auth {
		Identity::Anonymous => Err(StatusCode::UNAUTHORIZED.into()),
		Identity::Remote { .. } => Err(StatusCode::NOT_IMPLEMENTED.into()),
		Identity::Local { id: uid, .. } => if ctx.uid(&id) == uid {
			tracing::debug!("processing new local activity: {}", serde_json::to_string(&activity).unwrap_or_default());
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

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Announce))) =>
					Ok(CreationResult(ctx.announce(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Accept(AcceptType::Accept)))) =>
					Ok(CreationResult(ctx.accept(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Reject(RejectType::Reject)))) =>
					Ok(CreationResult(ctx.reject(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Undo))) =>
					Ok(CreationResult(ctx.undo(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Delete))) =>
					Ok(CreationResult(ctx.delete(uid, activity).await?)),

				Some(BaseType::Object(ObjectType::Activity(ActivityType::Update))) =>
					Ok(CreationResult(ctx.update(uid, activity).await?)),

				Some(_) => Err(StatusCode::NOT_IMPLEMENTED.into()),
			}
		} else {
			Err(StatusCode::FORBIDDEN.into())
		}
	}
}