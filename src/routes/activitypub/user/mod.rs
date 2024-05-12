pub mod inbox;

pub mod outbox;

pub mod following;

use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, SelectColumns};

use apb::{ActorMut, Node};
use crate::{errors::UpubError, model::{self, user}, server::{auth::AuthIdentity, fetcher::Fetcher, Context}, url};

use super::{jsonld::LD, JsonLD, TryFetch};


pub async fn view(
	State(ctx) : State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(id): Path<String>,
	Query(query): Query<TryFetch>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let mut uid = ctx.uri("users", id.clone());
	if auth.is_local() && query.fetch && !ctx.is_local(&uid) {
		if id.starts_with('@') {
			if let Some((user, host)) = id.replacen('@', "", 1).split_once('@') {
				uid = ctx.webfinger(user, host).await?;
			}
		}
		ctx.fetch_user(&uid).await?;
	}

	let (followed_by_me, following_me) = match auth.my_id() {
		None => (None, None),
		Some(my_id) => {
			// TODO these two queries are fast because of indexes but still are 2 subqueries for each
			// user GET, not even parallelized... should really add these as joins on the main query, so
			// that it's one roundtrip only
			let followed_by_me = model::relation::Entity::find()
				.filter(model::relation::Column::Follower.eq(my_id))
				.filter(model::relation::Column::Following.eq(&uid))
				.select_only()
				.select_column(model::relation::Column::Follower)
				.into_tuple::<String>()
				.one(ctx.db())
				.await?
				.map(|_| true);

			let following_me = model::relation::Entity::find()
				.filter(model::relation::Column::Following.eq(my_id))
				.filter(model::relation::Column::Follower.eq(&uid))
				.select_only()
				.select_column(model::relation::Column::Follower)
				.into_tuple::<String>()
				.one(ctx.db())
				.await?
				.map(|_| true);

			(followed_by_me, following_me)
		},
	};

	match user::Entity::find_by_id(&uid)
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
				.set_followed_by_me(followed_by_me);

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

