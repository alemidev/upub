mod inbox;
pub use inbox::inbox;

mod outbox;
pub use outbox::outbox;

mod following;
pub use following::follow___;

use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{activitystream::{object::{actor::ActorMut, collection::{CollectionMut, CollectionType}, document::{DocumentMut, DocumentType}, ObjectMut}, BaseMut, Node}, model::{self, user}, server::Context, url};

use super::{jsonld::LD, JsonLD};

pub async fn list(State(_db) : State<Arc<DatabaseConnection>>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	todo!()
}

pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match user::Entity::find_by_id(ctx.uid(id.clone()))
		.find_also_related(model::config::Entity)
		.one(ctx.db()).await
	{
		// local user
		Ok(Some((user, Some(cfg)))) => {
			Ok(JsonLD(serde_json::Value::new_object()
				.set_id(Some(&user.id))
				.set_actor_type(Some(user.actor_type))
				.set_name(user.name.as_deref())
				.set_summary(user.summary.as_deref())
				.set_icon(Node::maybe_object(user.icon.map(|i|
					serde_json::Value::new_object()
						.set_document_type(Some(DocumentType::Image))
						.set_url(Node::link(i.clone()))
				)))
				.set_image(Node::maybe_object(user.image.map(|i|
					serde_json::Value::new_object()
						.set_document_type(Some(DocumentType::Image))
						.set_url(Node::link(i.clone()))
				)))
				.set_published(Some(user.created))
				.set_preferred_username(Some(&user.preferred_username))
				.set_inbox(Node::link(url!(ctx, "/users/{id}/inbox")))
				.set_outbox(Node::link(url!(ctx, "/users/{id}/outbox")))
				.set_following(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/following")))
						.set_collection_type(Some(CollectionType::OrderedCollection))
						.set_total_items(if cfg.show_following_count { Some(0 /* user.following_count TODO */) } else { None })
						.set_first(Node::link(url!(ctx, "/users/{id}/following?page=true")))
				))
				.set_followers(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/followers")))
						.set_collection_type(Some(CollectionType::OrderedCollection))
						.set_total_items(if cfg.show_followers_count { Some(0 /* user.followers_count TODO */) } else { None })
						.set_first(Node::link(url!(ctx, "/users/{id}/followers?page=true")))
				))
				// .set_public_key(user.public_key) // TODO
				.set_discoverable(Some(true))
				.set_endpoints(None)
			))
		},
		// remote user
		Ok(Some((_user, None))) => {
			Err(StatusCode::NOT_IMPLEMENTED)
		},
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for user: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}


