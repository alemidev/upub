mod inbox;
pub use inbox::inbox;

mod outbox;
pub use outbox::outbox;

mod following;
pub use following::follow___;

use axum::{extract::{Path, State}, http::StatusCode};
use sea_orm::EntityTrait;

use crate::{activitystream::{object::{actor::ActorMut, collection::{CollectionMut, CollectionType}, document::{DocumentMut, DocumentType}, ObjectMut}, BaseMut, Node}, model::{self, user}, server::Context, url};

use super::{jsonld::LD, JsonLD};

pub fn ap_user(user: model::user::Model) -> serde_json::Value {
	serde_json::Value::new_object()
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
		.set_inbox(Node::maybe_link(user.inbox))
		.set_outbox(Node::maybe_link(user.outbox))
		.set_following(Node::object(
			serde_json::Value::new_object()
				.set_id(user.following.as_deref())
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_total_items(Some(user.following_count as u64))
		))
		.set_followers(Node::object(
			serde_json::Value::new_object()
				.set_id(user.followers.as_deref())
				.set_collection_type(Some(CollectionType::OrderedCollection))
				.set_total_items(Some(user.followers_count as u64))
		))
		// .set_public_key(user.public_key) // TODO
		.set_discoverable(Some(true))
		.set_endpoints(None)
}

pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match user::Entity::find_by_id(ctx.uid(id.clone()))
		.find_also_related(model::config::Entity)
		.one(ctx.db()).await
	{
		// local user
		Ok(Some((user, Some(cfg)))) => {
			Ok(JsonLD(ap_user(user.clone()) // ew ugly clone TODO
				.set_inbox(Node::link(url!(ctx, "/users/{id}/inbox")))
				.set_outbox(Node::link(url!(ctx, "/users/{id}/outbox")))
				.set_following(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/following")))
						.set_collection_type(Some(CollectionType::OrderedCollection))
						.set_total_items(if cfg.show_following_count { Some(user.following_count as u64) } else { None })
						.set_first(Node::link(url!(ctx, "/users/{id}/following?page=true")))
				))
				.set_followers(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/followers")))
						.set_collection_type(Some(CollectionType::OrderedCollection))
						.set_total_items(if cfg.show_followers_count { Some(user.followers_count as u64) } else { None })
						.set_first(Node::link(url!(ctx, "/users/{id}/followers?page=true")))
				))
				// .set_public_key(user.public_key) // TODO
				.ld_context()
			))
		},
		// remote user
		Ok(Some((user, None))) => Ok(JsonLD(ap_user(user).ld_context())),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for user: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

