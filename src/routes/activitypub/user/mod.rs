pub mod inbox;

pub mod outbox;

pub mod following;

use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, SelectColumns};

use apb::{ActorMut, CollectionMut, Node, Object, ObjectMut};
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
		Some((user_model, Some(cfg))) => {
			let mut user = user_model.ap()
				.set_inbox(Node::link(url!(ctx, "/users/{id}/inbox")))
				.set_outbox(Node::link(url!(ctx, "/users/{id}/outbox")))
				.set_following(Node::link(url!(ctx, "/users/{id}/following")))
				.set_followers(Node::link(url!(ctx, "/users/{id}/followers")));

			// TODO maybe this thing could be made as a single join, to avoid triple db roundtrip for
			// each fetch made by local users? it's indexed and fast but still...
			if let Some(my_id) = auth.my_id() {
				if !auth.is(&uid) {
					let followed_by_me = model::relation::Entity::find()
						.filter(model::relation::Column::Follower.eq(my_id))
						.filter(model::relation::Column::Following.eq(&uid))
						.select_column(model::relation::Column::Follower)
						.into_tuple::<String>()
						.all(ctx.db())
						.await?;

					user
						.audience()
						.update(|x| x.set_ordered_items(apb::Node::links(followed_by_me)));

					let following_me = model::relation::Entity::find()
						.filter(model::relation::Column::Following.eq(my_id))
						.filter(model::relation::Column::Follower.eq(&uid))
						.select_column(model::relation::Column::Following)
						.into_tuple::<String>()
						.all(ctx.db())
						.await?;
					
					user
						.generator()
						.update(|x| x.set_ordered_items(apb::Node::links(following_me)));
				}
			}

			if !auth.is(&uid) && !cfg.show_followers_count {
				user = user.set_audience(apb::Node::Empty);
			}

			if !auth.is(&uid) && !cfg.show_following_count {
				user = user.set_generator(apb::Node::Empty);
			}

			Ok(JsonLD(user.ld_context()))
		},
		// remote user TODDO doesn't work?
		Some((user, None)) => Ok(JsonLD(user.ap().ld_context())),
		None => Err(UpubError::not_found()),
	}
}

