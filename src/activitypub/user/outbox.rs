use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sea_orm::{EntityTrait, Order, QueryOrder, QuerySelect};

use crate::{activitypub::{jsonld::LD, JsonLD, Pagination}, activitystream::{object::{activity::ActivityMut, collection::{page::CollectionPageMut, CollectionMut, CollectionType}}, Base, BaseMut, BaseType, Node}, auth::{AuthIdentity, Identity}, model::{activity, object}, server::Context, url};

pub async fn get(
	State(ctx): State<Context>,
	Path(id): Path<String>,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	Ok(JsonLD(
		serde_json::Value::new_object()
			.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
			.set_collection_type(Some(CollectionType::OrderedCollection))
			.set_first(Node::link(url!(ctx, "/users/{id}/outbox/page")))
			.ld_context()
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

	match activity::Entity::find()
		.find_also_related(object::Entity)
		.order_by(activity::Column::Published, Order::Desc)
		.limit(limit)
		.offset(offset)
		.all(ctx.db()).await
	{
		Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
		Ok(items) => {
			Ok(JsonLD(
				serde_json::Value::new_object()
					// TODO set id, calculate uri from given args
					.set_id(Some(&url!(ctx, "/users/{id}/outbox/page?offset={offset}")))
					.set_collection_type(Some(CollectionType::OrderedCollectionPage))
					.set_part_of(Node::link(url!(ctx, "/users/{id}/outbox")))
					.set_next(Node::link(url!(ctx, "/users/{id}/outbox/page?offset={}", limit+offset)))
					.set_ordered_items(Node::array(
						items
							.into_iter()
							.map(|(a, o)| 
								super::super::activity::ap_activity(a)
									.set_object(Node::maybe_object(o.map(super::super::object::ap_object)))
							)
							.collect()
					))
					.ld_context()
			))
		},
	}
}

pub async fn post(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Json(activity): Json<serde_json::Value>,
	AuthIdentity(auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match auth {
		Identity::Anonymous => Err(StatusCode::UNAUTHORIZED),
		Identity::Server(_) => Err(StatusCode::NOT_IMPLEMENTED),
		Identity::User(uid) => if ctx.uid(id) == uid {
			match activity.base_type() {
				None => Err(StatusCode::BAD_REQUEST),
				Some(BaseType::Link(_)) => Err(StatusCode::UNPROCESSABLE_ENTITY),
				// Some(BaseType::Object(ObjectType::Note)) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Create))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Like))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Follow))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Undo))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Accept(AcceptType::Accept)))) => {
				// },
				// Some(BaseType::Object(ObjectType::Activity(ActivityType::Reject(RejectType::Reject)))) => {
				// },
				Some(_) => Err(StatusCode::NOT_IMPLEMENTED),
			}
		} else {
			Err(StatusCode::FORBIDDEN)
		}
	}
}
