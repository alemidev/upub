use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition, EntityTrait, JoinType, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait};

use apb::{server::Inbox, ActivityMut, ActivityType, Base, BaseType, ObjectType};
use crate::{errors::UpubError, model, routes::activitypub::{activity::ap_activity, jsonld::LD, object::ap_object, JsonLD, Pagination}, server::{auth::{AuthIdentity, Identity}, Context}, tools::ActivityWithObject, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match auth {
		Identity::Anonymous => Err(StatusCode::FORBIDDEN),
		Identity::Remote(_) => Err(StatusCode::FORBIDDEN),
		Identity::Local(user) => if ctx.uid(id.clone()) == user {
			Ok(JsonLD(ctx.ap_collection(&url!(ctx, "/users/{id}/inbox"), None).ld_context()))
		} else {
			Err(StatusCode::FORBIDDEN)
		},
	}
}

pub async fn page(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(page): Query<Pagination>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let uid = ctx.uid(id.clone());
	match auth {
		Identity::Anonymous => Err(StatusCode::FORBIDDEN.into()),
		Identity::Remote(_) => Err(StatusCode::FORBIDDEN.into()),
		Identity::Local(user) => if uid == user {
			let limit = page.batch.unwrap_or(20).min(50);
			let offset = page.offset.unwrap_or(0);
			let select = model::addressing::Entity::find()
				.filter(Condition::all().add(model::addressing::Column::Actor.eq(uid)))
				.order_by(model::addressing::Column::Published, Order::Asc)
				.select_only();

			match crate::tools::Prefixer::new(select)
				.add_columns(model::activity::Entity)
				.add_columns(model::object::Entity)
				.selector
				.join(JoinType::LeftJoin, model::activity::Relation::Addressing.def().rev())
				.join(JoinType::LeftJoin, model::object::Relation::Activity.def().rev())
				.limit(limit)
				.offset(offset)
				.into_model::<crate::tools::ActivityWithObject>()
				.all(ctx.db())
				.await
			{
				Ok(activities) => {
					Ok(JsonLD(
						ctx.ap_collection_page(
							&url!(ctx, "/users/{id}/inbox/page"),
							offset, limit,
							activities
								.into_iter()
								.map(|ActivityWithObject { activity, object }| {
									ap_activity(activity)
										.set_object(apb::Node::maybe_object(object.map(ap_object)))
								})
								.collect::<Vec<serde_json::Value>>()
						).ld_context()
					))
				},
				Err(e) => {
					tracing::error!("failed paginating user inbox for {id}: {e}");
					Err(StatusCode::INTERNAL_SERVER_ERROR.into())
				},
			}
		} else {
			Err(StatusCode::FORBIDDEN.into())
		},
	}
}

pub async fn post(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	Json(activity): Json<serde_json::Value>
) -> Result<(), UpubError> {
	match activity.base_type() {
		None => { Err(StatusCode::BAD_REQUEST.into()) },

		Some(BaseType::Link(_x)) => {
			tracing::warn!("skipping remote activity: {}", serde_json::to_string_pretty(&activity).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY.into()) // we could but not yet
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => {
			tracing::warn!("skipping unprocessable base activity: {}", serde_json::to_string_pretty(&activity).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY.into()) // won't ingest useless stuff
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Delete))) =>
			Ok(ctx.delete(activity).await?),

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) =>
			Ok(ctx.follow(activity).await?),

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Accept(_)))) =>
			Ok(ctx.accept(activity).await?),

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Reject(_)))) =>
			Ok(ctx.reject(activity).await?),

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) =>
			Ok(ctx.like(activity).await?),

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) =>
			Ok(ctx.create(activity).await?),

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Update))) =>
			Ok(ctx.update(activity).await?),

		Some(BaseType::Object(ObjectType::Activity(_x))) => {
			tracing::info!("received unimplemented activity on inbox: {}", serde_json::to_string_pretty(&activity).unwrap());
			Err(StatusCode::NOT_IMPLEMENTED.into())
		},

		Some(_x) => {
			tracing::warn!("ignoring non-activity object in inbox: {}", serde_json::to_string_pretty(&activity).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY.into())
		}
	}
}
