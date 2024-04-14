use apb::{server::Inbox, ActivityType, Base, BaseType, ObjectType};
use axum::{extract::{Query, State}, http::StatusCode, Json};
use sea_orm::{Order, QueryFilter, QueryOrder, QuerySelect};

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
		.order_by(model::addressing::Column::Published, Order::Desc)
		.limit(limit)
		.offset(offset)
		.into_model::<EmbeddedActivity>()
		.all(ctx.db())
		.await?;
	Ok(JsonLD(
		ctx.ap_collection_page(
			&url!(ctx, "/inbox/page"),
			offset, limit,
			activities
				.into_iter()
				.map(|x| x.into())
				.collect()
		).ld_context()
	))
}



pub async fn post(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Json(activity): Json<serde_json::Value>
) -> crate::Result<()> {
	match auth {
		Identity::Remote(_server) => {},
		Identity::Local(_user) => return Err(UpubError::forbidden()),
		Identity::Anonymous => return Err(UpubError::unauthorized()),
	}
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
