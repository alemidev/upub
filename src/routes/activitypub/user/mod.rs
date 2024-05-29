pub mod inbox;

pub mod outbox;

pub mod following;

use axum::extract::{Path, Query, State};

use apb::{ActorMut, EndpointsMut, Node};
use crate::{errors::UpubError, model, server::{auth::AuthIdentity, builders::AnyQuery, fetcher::Fetcher, Context}, url};

use super::{jsonld::LD, JsonLD, TryFetch};


pub async fn view(
	State(ctx) : State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(id): Path<String>,
	Query(query): Query<TryFetch>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let mut uid = ctx.uid(&id);
	if auth.is_local() {
		if id.starts_with('@') {
			if let Some((user, host)) = id.replacen('@', "", 1).split_once('@') {
				uid = ctx.webfinger(user, host).await?;
			}
		}
		if query.fetch && !ctx.is_local(&uid) {
			ctx.fetch_user(&uid).await?;
		}
	}
	let internal_uid = model::actor::Entity::ap_to_internal(&uid, ctx.db()).await?;

	let (followed_by_me, following_me) = match auth.my_id() {
		None => (None, None),
		Some(my_id) => {
			// TODO these two queries are fast because of indexes but still are 2 subqueries for each
			// user GET, not even parallelized... should maybe add these as joins on the main query? so
			// that it's one roundtrip only
			let followed_by_me = model::relation::Entity::is_following(my_id, internal_uid).any(ctx.db()).await?;
			let following_me = model::relation::Entity::is_following(internal_uid, my_id).any(ctx.db()).await?;
			(Some(followed_by_me), Some(following_me))
		},
	};

	match model::actor::Entity::find_by_ap_id(&uid)
		.find_also_related(model::config::Entity)
		.one(ctx.db()).await?
	{
		// local user
		Some((user_model, Some(cfg))) => {
			let mut user = user_model.ap()
				.set_inbox(Node::link(url!(ctx, "/users/{id}/inbox")))
				.set_outbox(Node::link(url!(ctx, "/users/{id}/outbox")))
				.set_following(Node::link(url!(ctx, "/users/{id}/following")))
				.set_followers(Node::link(url!(ctx, "/users/{id}/followers")))
				.set_following_me(following_me)
				.set_followed_by_me(followed_by_me)
				.set_endpoints(Node::object(
					serde_json::Value::new_object()
						.set_shared_inbox(Some(&url!(ctx, "/inbox")))
						.set_proxy_url(Some(&url!(ctx, "/proxy")))
				));

			if !auth.is(&uid) && !cfg.show_followers_count {
				user = user.set_followers_count(None);
			}

			if !auth.is(&uid) && !cfg.show_following_count {
				user = user.set_following_count(None);
			}

			Ok(JsonLD(user.ld_context()))
		},
		// remote user
		Some((user_model, None)) => Ok(JsonLD(
			user_model.ap()
				.set_following_me(following_me)
				.set_followed_by_me(followed_by_me)
				.ld_context()
		)),
		None => Err(UpubError::not_found()),
	}
}

