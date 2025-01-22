pub mod inbox;
pub mod outbox;
pub mod likes;
pub mod following;
pub mod notifications;
// pub mod audience;

use axum::extract::{Path, Query, State};

use apb::{LD, ActorMut, Node, ObjectMut};
use upub::{ext::AnyQuery, model, traits::Fetcher, Context};

use crate::{builders::JsonLD, ApiError, AuthIdentity};

use super::TryFetch;


pub async fn view(
	State(ctx) : State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(id): Path<String>,
	Query(query): Query<TryFetch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let mut uid = ctx.uid(&id);
	if auth.is_local() {
		if id.starts_with('@') {
			if let Some((user, host)) = id.replacen('@', "", 1).split_once('@') {
				if let Some(webfinger) = ctx.webfinger(user, host).await? {
					uid = webfinger;
				}
			}
		}
		if query.fetch && !ctx.is_local(&uid) {
			ctx.fetch_user(&uid, ctx.db()).await?;
		}
	}
	let internal_uid = model::actor::Entity::ap_to_internal(&uid, ctx.db())
		.await?
		.ok_or_else(ApiError::not_found)?;

	let (followed_by_me, following_me) = match auth.my_id() {
		None => (None, None),
		Some(my_id) => {
			// TODO these two queries are fast because of indexes but still are 2 subqueries for each
			// user GET, not even parallelized... should maybe add these as joins on the main query? so
			// that it's one roundtrip only
			let followed_by_me = upub::Query::related(Some(my_id), Some(internal_uid), false).any(ctx.db()).await?;
			let following_me = upub::Query::related(Some(internal_uid), Some(my_id), false).any(ctx.db()).await?;
			(Some(followed_by_me), Some(following_me))
		},
	};

	match model::actor::Entity::find_by_ap_id(&uid)
		.find_also_related(model::config::Entity)
		.one(ctx.db()).await?
	{
		// local user
		Some((user_model, Some(cfg))) => {
			let (followers, following) = (user_model.followers_count, user_model.following_count);
			let mut user = ctx.ap(user_model)
				.set_following_me(following_me)
				.set_followed_by_me(followed_by_me)
				.set_manually_approves_followers(Some(!cfg.accept_follow_requests));

			if auth.is(&uid) {
				user = user.set_notifications(Node::link(upub::url!(ctx, "/actors/{id}/notifications")));
			}

			if auth.is(&uid) || cfg.show_followers_count {
				user = user.set_followers_count(Some(u64::try_from(followers).unwrap_or(0)));
			}

			if auth.is(&uid) || cfg.show_following_count {
				user = user.set_following_count(Some(u64::try_from(following).unwrap_or(0)));
			}

			// TODO this is known "magically" !! very tight coupling ouchhh
			if !ctx.cfg().instance.frontend.is_empty() {
				user = user.set_url(Node::link(format!("{}/actors/{id}", ctx.cfg().instance.frontend)));
			}

			Ok(JsonLD(user.ld_context()))
		},
		// remote user
		Some((user_model, None)) => Ok(JsonLD(
			ctx.ap(user_model)
				.set_following_me(following_me)
				.set_followed_by_me(followed_by_me)
				.ld_context()
		)),
		None => Err(crate::ApiError::not_found()),
	}
}

