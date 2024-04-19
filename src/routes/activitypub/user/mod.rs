pub mod inbox;

pub mod outbox;

pub mod following;

use axum::extract::{Path, Query, State};
use sea_orm::EntityTrait;

use apb::{ActorMut, BaseMut, CollectionMut, Node};
use crate::{errors::UpubError, model::{self, user}, server::{auth::AuthIdentity, fetcher::Fetcher, Context}, url};

use super::{jsonld::LD, JsonLD, TryFetch};


pub async fn view(
	State(ctx) : State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(id): Path<String>,
	Query(query): Query<TryFetch>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let uid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.uid(id.clone())
	};
	if auth.is_local() && query.fetch && !ctx.is_local(&uid) {
		ctx.fetch_user(&uid).await?;
	}
	match user::Entity::find_by_id(&uid)
		.find_also_related(model::config::Entity)
		.one(ctx.db()).await?
	{
		// local user
		Some((user, Some(cfg))) => {
			Ok(JsonLD(user.clone().ap() // ew ugly clone TODO
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
							if auth.is_local_user(&user.id) || cfg.show_following {
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
							if auth.is_local_user(&user.id) || cfg.show_followers {
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
		Some((user, None)) => Ok(JsonLD(user.ap().ld_context())),
		None => Err(UpubError::not_found()),
	}
}

