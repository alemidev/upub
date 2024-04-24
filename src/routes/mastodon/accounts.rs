use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use mastodon_async_entities::{account::{Account, AccountId}, status::Status};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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

pub struct StatusesQuery {
	/// All results returned will be lesser than this ID. In effect, sets an upper bound on results.
	pub max_id: String,
	/// All results returned will be greater than this ID. In effect, sets a lower bound on results.
	pub since_id: String,
	/// Returns results immediately newer than this ID. In effect, sets a cursor at this ID and paginates forward.
	pub min_id: String,
	/// Maximum number of results to return. Defaults to 20 statuses. Max 40 statuses.
	pub limit: i32,
	/// Filter out statuses without attachments.
	pub only_media: bool,
	/// Filter out statuses in reply to a different account.
	pub exclude_replies: bool,
	/// Filter out boosts from the response.
	pub exclude_reblogs: bool,
	/// Filter for pinned statuses only. Defaults to false, which includes all statuses. Pinned statuses do not receive special priority in the order of the returned results.
	pub pinned: bool,
	/// Filter for statuses using a specific hashtag.
	pub tagged: String,
}

pub async fn statuses(
	State(ctx): State<Context>,
	AuthIdentity(auth): AuthIdentity,
	Path(id): Path<String>,
	Query(_query): Query<StatusesQuery>,
) -> Result<Json<Vec<Status>>, StatusCode> {
	let uid = ctx.uid(id);
	model::addressing::Entity::find_addressed()
		.filter(model::activity::Column::Actor.eq(uid))
		.filter(auth.filter_condition());

	todo!()
}
