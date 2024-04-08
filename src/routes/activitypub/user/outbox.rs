use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{EntityTrait, Order, QueryOrder, QuerySelect};

use apb::{AcceptType, ActivityMut, ActivityType, Base, BaseType, Node, ObjectType, RejectType};
use crate::{routes::activitypub::{jsonld::LD, APOutbox, CreationResult, JsonLD, Pagination}, server::auth::{AuthIdentity, Identity}, errors::UpubError, model, server::Context, url};

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
	AuthIdentity(_auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	// let mut conditions = Condition::any()
	// 	.add(model::addressing::Column::Actor.eq(PUBLIC_TARGET));

	// if let Identity::User(ref x) = auth {
	// 	conditions = conditions.add(model::addressing::Column::Actor.eq(x));
	// }

	// if let Identity::Server(ref x) = auth {
	// 	conditions = conditions.add(model::addressing::Column::Server.eq(x));
	// }

	match model::activity::Entity::find()
		.find_also_related(model::object::Entity)
		.order_by(model::activity::Column::Published, Order::Desc)
		.limit(limit)
		.offset(offset)
		.all(ctx.db()).await
	{
		Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
		Ok(items) => {
			Ok(JsonLD(
				ctx.ap_collection_page(
					&url!(ctx, "/users/{id}/outbox/page"),
					offset, limit,
					items
						.into_iter()
						.map(|(a, o)| {
							let oid = a.object.clone();
							super::super::activity::ap_activity(a)
								.set_object(match o {
									Some(o) => Node::object(super::super::object::ap_object(o)),
									None    => Node::maybe_link(oid),
								})
						})
						.collect()
				).ld_context()
			))
		},
	}
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
