use std::sync::Arc;

use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{sea_query::Expr, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, SelectColumns};

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

pub async fn followers(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<super::Pagination>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	if let Some(true) = page.page {
		match model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Following.eq(id.clone())))
			.select_column(model::relation::Column::Follower)
			.limit(limit) // TODO allow customizing, with boundaries
			.offset(page.offset.unwrap_or(0))
			.all(ctx.db()).await
		{
			Err(e) => {
				tracing::error!("error queriying who {id} is following: {e}");
				Err(StatusCode::INTERNAL_SERVER_ERROR)
			},
			Ok(following) => {
				Ok(JsonLD(
					serde_json::Value::new_object()
						.set_collection_type(Some(CollectionType::OrderedCollectionPage))
						.set_part_of(Node::link(url!(ctx, "/users/{id}/followers")))
						.set_next(Node::link(url!(ctx, "/users/{id}/followers?page=true&offset={}", offset+limit)))
						.set_ordered_items(Node::array(following.into_iter().map(|x| x.follower).collect()))
						.ld_context()
				))
			},
		}
	} else {
		let count = model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Following.eq(id.clone())))
			.count(ctx.db()).await.unwrap_or_else(|e| {
				tracing::error!("failed counting followers for {id}: {e}");
				0
			});
		Ok(JsonLD(
			serde_json::Value::new_object()
				.set_id(Some(&format!("{}/users/{}/followers", ctx.base(), id)))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_total_items(Some(count))
				.set_first(Node::link(format!("{}/users/{}/followers?page=true", ctx.base(), id)))
				.ld_context()
		))
	}
}

pub async fn following(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<super::Pagination>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	if let Some(true) = page.page {
		match model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Follower.eq(id.clone())))
			.select_column(model::relation::Column::Following)
			.limit(limit) // TODO allow customizing, with boundaries
			.offset(page.offset.unwrap_or(0))
			.all(ctx.db()).await
		{
			Err(e) => {
				tracing::error!("error queriying who {id} is following: {e}");
				Err(StatusCode::INTERNAL_SERVER_ERROR)
			},
			Ok(following) => {
				Ok(JsonLD(
					serde_json::Value::new_object()
						.set_collection_type(Some(CollectionType::OrderedCollectionPage))
						.set_part_of(Node::link(url!(ctx, "/users/{id}/following")))
						.set_next(Node::link(url!(ctx, "/users/{id}/following?page=true&offset={}", offset+limit)))
						.set_ordered_items(Node::array(following.into_iter().map(|x| x.following).collect()))
						.ld_context()
				))
			},
		}
	} else {
		let count = model::relation::Entity::find()
			.filter(Condition::all().add(model::relation::Column::Follower.eq(id.clone())))
			.count(ctx.db()).await.unwrap_or_else(|e| {
				tracing::error!("failed counting following for {id}: {e}");
				0
			});
		Ok(JsonLD(
			serde_json::Value::new_object()
				.set_id(Some(&format!("{}/users/{}/following", ctx.base(), id)))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_total_items(Some(count))
				.set_first(Node::link(format!("{}/users/{}/following?page=true", ctx.base(), id)))
				.ld_context()
		))
	}
}

pub async fn outbox(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<super::Pagination>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);
	if let Some(true) = page.page {
		match activity::Entity::find()
			.find_also_related(object::Entity)
			.order_by(activity::Column::Published, Order::Desc)
			.limit(limit)
			.offset(offset)
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
						.ld_context()
				))
			},
		}

	} else {
		Ok(JsonLD(
			serde_json::Value::new_object()
				.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_first(Node::link(url!(ctx, "/users/{id}/outbox?page=true")))
				.ld_context()
		))
	}
}

pub async fn inbox(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	Json(object): Json<serde_json::Value>
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	tracing::info!("received object on inbox: {}", serde_json::to_string_pretty(&object).unwrap());
	match object.base_type() {
		None => { Err(StatusCode::BAD_REQUEST) },
		Some(BaseType::Link(_x)) => Err(StatusCode::UNPROCESSABLE_ENTITY), // we could but not yet
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => Err(StatusCode::UNPROCESSABLE_ENTITY), // won't ingest useless stuff
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => { Ok(JsonLD(serde_json::Value::Null)) },
		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => {
			let aid = object.actor().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			let oid = object.object().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			let like = model::like::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				actor: sea_orm::Set(aid.clone()),
				likes: sea_orm::Set(oid.clone()),
			};
			match model::like::Entity::insert(like).exec(ctx.db()).await {
				Err(sea_orm::DbErr::RecordNotInserted) => Err(StatusCode::NOT_MODIFIED),
				Err(e) => {
					tracing::error!("unexpected error procesing like from {aid} to {oid}: {e}");
					Err(StatusCode::INTERNAL_SERVER_ERROR)
				}
				Ok(_) => {
					match model::object::Entity::update_many()
						.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
						.filter(model::object::Column::Id.eq(oid.clone()))
						.exec(ctx.db())
						.await
					{
						Err(e) => {
							tracing::error!("unexpected error incrementing object {oid} like counter: {e}");
							Err(StatusCode::INTERNAL_SERVER_ERROR)
						},
						Ok(_) => Ok(JsonLD(serde_json::Value::Null)),
					}
				},
			}
		},
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
