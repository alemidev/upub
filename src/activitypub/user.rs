use std::sync::Arc;

use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, Order, QueryFilter, QueryOrder, QuerySelect};

use crate::{activitystream::{object::{activity::{Activity, ActivityMut, ActivityType}, collection::{page::CollectionPageMut, CollectionMut, CollectionType}, ObjectType}, Base, BaseMut, BaseType, Node}, model::{self, activity, object, user}, server::Context, url};

use super::{jsonld::LD, JsonLD};

pub async fn list(State(_db) : State<Arc<DatabaseConnection>>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	todo!()
}

pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match user::Entity::find_by_id(ctx.uid(id)).one(ctx.db()).await {
		Ok(Some(user)) => Ok(JsonLD(user.underlying_json_object().ld_context())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for user: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

pub async fn outbox(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<super::Page>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	if let Some(true) = page.page {

		// find requested recent post, to filter based on its date (use now() as fallback)
		let before = if let Some(before) = page.max_id {
			match model::activity::Entity::find_by_id(ctx.aid(before))
				.one(ctx.db()).await
			{
				Ok(None) => return Err(StatusCode::NOT_FOUND),
				Ok(Some(x)) => x.published,
				Err(e) => {
					tracing::error!("could not fetch activity from db: {e}");
					chrono::Utc::now()
				},
			}
		} else { chrono::Utc::now() };

		match activity::Entity::find()
			.filter(Condition::all().add(activity::Column::Published.lt(before)))
			.find_also_related(object::Entity)
			.order_by(activity::Column::Published, Order::Desc)
			.limit(20) // TODO allow customizing, with boundaries
			.all(ctx.db()).await
		{
			Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
			Ok(items) => {
				let next = ctx.id(items.last().map(|(a, _o)| a.id.as_str()).unwrap_or("").to_string());
				let items = items
					.into_iter()
					.map(|(a, o)| a.underlying_json_object().set_object(Node::maybe_object(o)))
					.collect();
				Ok(JsonLD(
					serde_json::Value::new_object()
						// TODO set id, calculate uri from given args
						.set_collection_type(Some(CollectionType::OrderedCollectionPage))
						.set_part_of(Node::link(url!(ctx, "/users/{id}/outbox")))
						.set_next(Node::link(url!(ctx, "/users/{id}/outbox?page=true&max_id={next}")))
						.set_ordered_items(Node::array(items))
				))
			},
		}

	} else {
		Ok(JsonLD(
			serde_json::Value::new_object()
				.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_first(Node::link(url!(ctx, "/users/{id}/outbox?page=true")))
		))
	}
}

pub async fn inbox(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	Json(object): Json<serde_json::Value>
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match object.base_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(BaseType::Link(_x)) => Err(StatusCode::UNPROCESSABLE_ENTITY), // we could but not yet
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => Err(StatusCode::UNPROCESSABLE_ENTITY),
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => { todo!() },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
			let Ok(activity_entity) = activity::Model::new(&object) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Node::Object(obj) = object.object() else {
				// TODO we could process non-embedded activities or arrays but im lazy rn
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			let Ok(obj_entity) = object::Model::new(&*obj) else {
				return Err(StatusCode::UNPROCESSABLE_ENTITY);
			};
			object::Entity::insert(obj_entity.into_active_model())
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			activity::Entity::insert(activity_entity.into_active_model())
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(JsonLD(serde_json::Value::Null)) // TODO hmmmmmmmmmmm not the best value to return....
		},
		Some(BaseType::Object(ObjectType::Activity(_x))) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(_x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}
