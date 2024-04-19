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
		Some((user, Some(_cfg))) => {
			Ok(JsonLD(user.clone().ap() // ew ugly clone TODO
				.set_inbox(Node::link(url!(ctx, "/users/{id}/inbox"))) // TODO unread activities as count
				.set_outbox(Node::object(
					serde_json::Value::new_object()
						.set_id(Some(&url!(ctx, "/users/{id}/outbox")))
						.set_collection_type(Some(apb::CollectionType::OrderedCollection))
						.set_first(Node::link(url!(ctx, "/users/{id}/outbox/page")))
						.set_total_items(Some(user.statuses_count as u64))
				))
				.set_following(Node::link(url!(ctx, "/users/{id}/following")))
				.set_followers(Node::link(url!(ctx, "/users/{id}/followers")))
				.ld_context()
			))
		},
		// remote user TODDO doesn't work?
		Some((user, None)) => Ok(JsonLD(user.ap().ld_context())),
		None => Err(UpubError::not_found()),
	}
}

