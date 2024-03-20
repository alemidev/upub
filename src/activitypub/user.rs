use std::sync::Arc;

use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, Order, QueryFilter, QueryOrder, QuerySelect};

use crate::{activitystream::{self, object::{activity::{Activity, ActivityType}, collection::{page::CollectionPageMut, CollectionMut, CollectionType}, ObjectType}, Base, BaseMut, BaseType, Node}, model::{self, activity, object, user}, server::Context, url};

pub async fn list(State(_db) : State<Arc<DatabaseConnection>>) -> Result<Json<serde_json::Value>, StatusCode> {
	todo!()
}

pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
	match user::Entity::find_by_id(ctx.uid(id)).one(ctx.db()).await {
		Ok(Some(user)) => Ok(Json(user.underlying_json_object())),
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
) -> Result<Json<serde_json::Value>, StatusCode> {
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
			.order_by(activity::Column::Published, Order::Desc)
			.limit(20) // TODO allow customizing, with boundaries
			.all(ctx.db()).await
		{
			Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
			Ok(items) => {
				let next = ctx.id(items.last().map(|x| x.id.as_str()).unwrap_or("").to_string());
				let items = items
					.into_iter()
					.map(|i| i.underlying_json_object())
					.collect();
				let mut obj = activitystream::object();
				obj
					// TODO set id, calculate uri from given args
					.set_collection_type(Some(CollectionType::OrderedCollectionPage))
					.set_part_of(Node::link(&url!(ctx, "/users/{id}/outbox")))
					.set_next(Node::link(&url!(ctx, "/users/{id}/outbox?page=true&max_id={next}")))
					.set_ordered_items(Node::array(items));
				Ok(Json(obj))
			},
		}

	} else {
		let mut obj = crate::activitystream::object();
		obj
			.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
			.set_collection_type(Some(CollectionType::OrderedCollection))
			.set_first(Node::link(&url!(ctx, "/users/{id}/outbox?page=true")));
		Ok(Json(obj.underlying_json_object()))
	}
}

pub async fn inbox(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	Json(object): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
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
			Ok(Json(serde_json::Value::Null)) // TODO hmmmmmmmmmmm not the best value to return....
		},
		Some(BaseType::Object(ObjectType::Activity(_x))) => { Err(StatusCode::NOT_IMPLEMENTED) },
		Some(_x) => { Err(StatusCode::UNPROCESSABLE_ENTITY) }
	}
}
