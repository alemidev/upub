pub mod inbox;

pub mod outbox;

pub mod following;

use axum::extract::{Path, State};
use sea_orm::EntityTrait;

use apb::{ActorMut, BaseMut, CollectionMut, DocumentMut, DocumentType, Node, ObjectMut, PublicKeyMut};
use crate::{errors::UpubError, model::{self, user}, server::{auth::AuthIdentity, Context}, url};

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
		.set_following(Node::maybe_link(user.following))
		.set_followers(Node::maybe_link(user.followers))
		.set_public_key(Node::object(
			serde_json::Value::new_object()
				.set_id(Some(&format!("{}#main-key", user.id)))
				.set_owner(Some(&user.id))
				.set_public_key_pem(&user.public_key)
		))
		.set_discoverable(Some(true))
		.set_endpoints(Node::Empty)
}

pub async fn view(
	State(ctx) : State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(id): Path<String>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let uid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.uid(id.clone())
	};
	match user::Entity::find_by_id(uid)
		.find_also_related(model::config::Entity)
		.one(ctx.db()).await?
	{
		// local user
		Some((user, Some(cfg))) => {
			Ok(JsonLD(ap_user(user.clone()) // ew ugly clone TODO
				.set_inbox(Node::link(url!(ctx, "/users/{id}/inbox"))) // TODO unread activities as count
				.set_outbox(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
						.set_collection_type(Some(apb::CollectionType::OrderedCollection))
						.set_first(Node::link(url!(ctx, "/users/{id}/outbox/page")))
						.set_total_items(Some(user.statuses_count as u64))
				))
				.set_following(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/following")))
						.set_collection_type(Some(apb::CollectionType::OrderedCollection))
						.set_first(Node::link(url!(ctx, "/users/{id}/following/page")))
						.set_total_items(
							if auth.is_user(&user.id) || cfg.show_following {
								Some(user.following_count as u64)
							} else {
								None
							}
						)
				))
				.set_followers(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/followers")))
						.set_collection_type(Some(apb::CollectionType::OrderedCollection))
						.set_first(Node::link(url!(ctx, "/users/{id}/followers/page")))
						.set_total_items(
							if auth.is_user(&user.id) || cfg.show_followers {
								Some(user.followers_count as u64)
							} else {
								None
							}
						)
				))
				// .set_public_key(user.public_key) // TODO
				.ld_context()
			))
		},
		// remote user TODDO doesn't work?
		Some((user, None)) => Ok(JsonLD(ap_user(user).ld_context())),
		None => Err(UpubError::not_found()),
	}
}

