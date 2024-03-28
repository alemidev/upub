use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{sea_query::Expr, ColumnTrait, Condition, EntityTrait, IntoActiveModel, Order, QueryFilter, QueryOrder, QuerySelect, Set};

use crate::{activitypub::{activity::ap_activity, jsonld::LD, JsonLD, Pagination, PUBLIC_TARGET}, activitystream::{object::{activity::{Activity, ActivityType}, collection::{page::CollectionPageMut, CollectionMut, CollectionType}, Addressed, Object, ObjectType}, Base, BaseMut, BaseType, Node}, auth::{AuthIdentity, Identity}, errors::{LoggableError, UpubError}, model, server::Context, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match auth {
		Identity::Anonymous => Err(StatusCode::FORBIDDEN),
		Identity::Remote(_) => Err(StatusCode::FORBIDDEN),
		Identity::Local(user) => if ctx.uid(id.clone()) == user {
			Ok(JsonLD(serde_json::Value::new_object()
				.set_id(Some(&url!(ctx, "/users/{id}/inbox")))
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_first(Node::link(url!(ctx, "/users/{id}/inbox/page")))
				.ld_context()
			))
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
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let uid = ctx.uid(id.clone());
	match auth {
		Identity::Anonymous => Err(StatusCode::FORBIDDEN),
		Identity::Remote(_) => Err(StatusCode::FORBIDDEN),
		Identity::Local(user) => if uid == user {
			let limit = page.batch.unwrap_or(20).min(50);
			let offset = page.offset.unwrap_or(0);
			match model::addressing::Entity::find()
				.filter(Condition::any()
					.add(model::addressing::Column::Actor.eq(PUBLIC_TARGET))
					.add(model::addressing::Column::Actor.eq(uid))
				)
				.order_by(model::addressing::Column::Published, Order::Asc)
				.find_also_related(model::activity::Entity)
				.limit(limit)
				.offset(offset)
				.all(ctx.db())
				.await
			{
				Ok(activities) => {
					Ok(JsonLD(serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/inbox/page?offset={offset}")))
						.set_collection_type(Some(CollectionType::OrderedCollectionPage))
						.set_part_of(Node::link(url!(ctx, "/users/{id}/inbox")))
						.set_next(Node::link(url!(ctx, "/users/{id}/inbox/page?offset={}", offset+limit)))
						.set_ordered_items(Node::array(
							activities
								.into_iter()
								.filter_map(|(_, a)| Some(ap_activity(a?)))
								.collect::<Vec<serde_json::Value>>()
						))
						.ld_context()
					))
				},
				Err(e) => {
					tracing::error!("failed paginating user inbox for {id}: {e}");
					Err(StatusCode::INTERNAL_SERVER_ERROR)
				},
			}
		} else {
			Err(StatusCode::FORBIDDEN)
		},
	}
}

pub async fn post(
	State(ctx): State<Context>,
	Path(_id): Path<String>,
	Json(object): Json<serde_json::Value>
) -> Result<(), UpubError> {
	match object.base_type() {
		None => { Err(StatusCode::BAD_REQUEST.into()) },

		Some(BaseType::Link(_x)) => {
			tracing::warn!("skipping remote activity: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY.into()) // we could but not yet
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Activity))) => {
			tracing::warn!("skipping unprocessable base activity: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY.into()) // won't ingest useless stuff
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Delete))) => {
			// TODO verify the signature before just deleting lmao
			let oid = object.object().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			// TODO maybe we should keep the tombstone?
			model::user::Entity::delete_by_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from users");
			model::activity::Entity::delete_by_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from activities");
			model::object::Entity::delete_by_id(&oid).exec(ctx.db()).await.info_failed("failed deleting from objects");
			Ok(())
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => {
			let activity_targets = object.addressed();
			let activity_entity = model::activity::Model::new(&object)?;
			let aid = activity_entity.id.clone();
			tracing::info!("{} wants to follow {}", activity_entity.actor, activity_entity.object.as_deref().unwrap_or("<no-one???>"));
			model::activity::Entity::insert(activity_entity.into_active_model())
				.exec(ctx.db()).await?;
			ctx.address_to(&aid, None, &activity_targets).await?;
			Ok(())
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Accept(_)))) => {
			// TODO what about TentativeAccept
			let activity_model = model::activity::Model::new(&object)?;
			let Some(follow_request_id) = activity_model.object else {
				return Err(StatusCode::BAD_REQUEST.into());
			};
			let Some(follow_activity) = model::activity::Entity::find_by_id(follow_request_id)
				.one(ctx.db()).await?
			else {
				return Err(StatusCode::NOT_FOUND.into());
			};
			if follow_activity.object.unwrap_or("".into()) != activity_model.actor {
				return Err(StatusCode::FORBIDDEN.into());
			}

			tracing::info!("{} accepted follow request by {}", activity_model.actor, follow_activity.actor);

			model::relation::Entity::insert(
				model::relation::ActiveModel {
					follower: Set(follow_activity.actor),
					following: Set(activity_model.actor),
					..Default::default()
				}
			).exec(ctx.db()).await?;

			ctx.address_to(&activity_model.id, None, &object.addressed()).await?;
			Ok(())
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Reject(_)))) => {
			// TODO what about TentativeReject?
			let activity_model = model::activity::Model::new(&object)?;
			let Some(follow_request_id) = activity_model.object else {
				return Err(StatusCode::BAD_REQUEST.into());
			};
			let Some(follow_activity) = model::activity::Entity::find_by_id(follow_request_id)
				.one(ctx.db()).await?
			else {
				return Err(StatusCode::NOT_FOUND.into());
			};
			if follow_activity.object.unwrap_or("".into()) != activity_model.actor {
				return Err(StatusCode::FORBIDDEN.into());
			}
			tracing::info!("{} rejected follow request by {}", activity_model.actor, follow_activity.actor);
			ctx.address_to(&activity_model.id, None, &object.addressed()).await?;
			Ok(())
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => {
			let aid = object.actor().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			let oid = object.object().id().ok_or(StatusCode::BAD_REQUEST)?.to_string();
			let like = model::like::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				actor: sea_orm::Set(aid.clone()),
				likes: sea_orm::Set(oid.clone()),
				date: sea_orm::Set(chrono::Utc::now()),
			};
			match model::like::Entity::insert(like).exec(ctx.db()).await {
				Err(sea_orm::DbErr::RecordNotInserted) => Err(StatusCode::NOT_MODIFIED.into()),
				Err(sea_orm::DbErr::Exec(_)) => Err(StatusCode::NOT_MODIFIED.into()), // bad fix for sqlite
				Err(e) => {
					tracing::error!("unexpected error procesing like from {aid} to {oid}: {e}");
					Err(StatusCode::INTERNAL_SERVER_ERROR.into())
				}
				Ok(_) => {
					model::object::Entity::update_many()
						.col_expr(model::object::Column::Likes, Expr::col(model::object::Column::Likes).add(1))
						.filter(model::object::Column::Id.eq(oid.clone()))
						.exec(ctx.db())
						.await?;
					tracing::info!("{} liked {}", aid, oid);
					Ok(())
				},
			}
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
			let activity_model = model::activity::Model::new(&object)?;
			let activity_targets = object.addressed();
			let Some(object_node) = object.object().get() else {
				// TODO we could process non-embedded activities or arrays but im lazy rn
				tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&object).unwrap());
				return Err(StatusCode::UNPROCESSABLE_ENTITY.into());
			};
			let object_model = model::object::Model::new(&object_node)?;
			let aid = activity_model.id.clone();
			let oid = object_model.id.clone();
			model::object::Entity::insert(object_model.into_active_model()).exec(ctx.db()).await?;
			model::activity::Entity::insert(activity_model.into_active_model()).exec(ctx.db()).await?;
			ctx.address_to(&aid, Some(&oid), &activity_targets).await?;
			tracing::info!("{} posted {}", aid, oid);
			Ok(())
		},

		Some(BaseType::Object(ObjectType::Activity(ActivityType::Update))) => {
			let activity_model = model::activity::Model::new(&object)?;
			let activity_targets = object.addressed();
			let Some(object_node) = object.object().get() else {
				// TODO we could process non-embedded activities or arrays but im lazy rn
				tracing::error!("refusing to process activity without embedded object: {}", serde_json::to_string_pretty(&object).unwrap());
				return Err(StatusCode::UNPROCESSABLE_ENTITY.into());
			};
			let aid = activity_model.id.clone();
			let Some(oid) = object_node.id().map(|x| x.to_string()) else {
				return Err(StatusCode::BAD_REQUEST.into());
			};
			model::activity::Entity::insert(activity_model.into_active_model()).exec(ctx.db()).await?;
			match object_node.object_type() {
				Some(ObjectType::Actor(_)) => {
					// TODO oof here is an example of the weakness of this model, we have to go all the way
					// back up to serde_json::Value because impl Object != impl Actor
					let actor_model = model::user::Model::new(&object_node.underlying_json_object())?;
					model::user::Entity::update(actor_model.into_active_model())
						.exec(ctx.db()).await?;
				},
				Some(ObjectType::Note) => {
					let object_model = model::object::Model::new(&object_node)?;
					model::object::Entity::update(object_model.into_active_model())
						.exec(ctx.db()).await?;
				},
				Some(t) => tracing::warn!("no side effects implemented for update type {t:?}"),
				None => tracing::warn!("empty type on embedded updated object"),
			}
			ctx.address_to(&aid, Some(&oid), &activity_targets).await?;
			tracing::info!("{} updated {}", aid, oid);
			Ok(())
		},

		Some(BaseType::Object(ObjectType::Activity(_x))) => {
			tracing::info!("received unimplemented activity on inbox: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::NOT_IMPLEMENTED.into())
		},

		Some(_x) => {
			tracing::warn!("ignoring non-activity object in inbox: {}", serde_json::to_string_pretty(&object).unwrap());
			Err(StatusCode::UNPROCESSABLE_ENTITY.into())
		}
	}
}
