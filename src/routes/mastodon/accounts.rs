use axum::{extract::{Path, State}, http::StatusCode, Json};
use mastodon_async_entities::account::{Account, AccountId};
use sea_orm::EntityTrait;

use crate::{model, server::{auth::AuthIdentity, Context}};

pub async fn view(
	State(ctx): State<Context>,
	AuthIdentity(_auth): AuthIdentity,
	Path(id): Path<String>
) -> Result<Json<Account>, StatusCode> {
	match model::user::Entity::find_by_id(ctx.uid(id))
		.find_also_related(model::config::Entity)
		.one(ctx.db())
		.await
	{
		Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Ok(Some((_x, None))) => Err(StatusCode::NOT_IMPLEMENTED), // TODO remote user
		Ok(Some((x, Some(cfg)))) => Ok(Json(Account {
			acct: x.preferred_username.clone(),
			avatar: x.icon.as_deref().unwrap_or("").to_string(),
			avatar_static: x.icon.unwrap_or_default(),
			created_at: time::OffsetDateTime::from_unix_timestamp(x.created.timestamp()).unwrap(),
			display_name: x.name.unwrap_or_default(),
			// TODO hide these maybe
			followers_count: x.followers_count as u64,
			following_count: x.following_count as u64,
			header: x.image.as_deref().unwrap_or("").to_string(),
			header_static: x.image.unwrap_or_default(),
			id: AccountId::new(x.id.clone()),
			locked: !cfg.accept_follow_requests,
			note: x.summary.unwrap_or_default(),
			statuses_count: 0, // TODO keep track in each user
			url: x.id,
			username: x.preferred_username,
			source: None,
			moved: None,
			fields: None, // TODO user fields
			bot: None,
		})),
	}
}