use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, SelectColumns, RelationTrait};

use upub::{model, Context};

use crate::{activitypub::Pagination, builders::JsonLD, ApiError, AuthIdentity, Identity};

pub async fn get<const OUTGOING: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let follow___ = if OUTGOING { "following" } else { "followers" };

	// TODO technically we could show remote instances following/followers count from that specific
	// instance, but that's annoying because means running a COUNT every time, so likely this will
	// keep answering 0

	let count = match model::actor::Entity::find_by_ap_id(&ctx.uid(&id))
		.find_also_related(model::config::Entity)
		.one(ctx.db())
		.await?
		.ok_or_else(ApiError::not_found)?
	{
		(user, Some(config)) => {
			let mut hide_count = if OUTGOING { !config.show_following } else { !config.show_followers };
			if hide_count {
				if let Some(internal) = auth.my_id() {
					if internal == user.internal {
						hide_count = false;
					}
				}
			}

			match (hide_count, OUTGOING) {
				(true, _) => 0,
				(false, true) => user.following_count,
				(false, false) => user.followers_count,
			}
		},
		(user, None) => {
			if !auth.is_local() {
				return Err(ApiError::forbidden());
			}
			if OUTGOING { user.following_count } else { user.followers_count }
		},
	};


	crate::builders::collection(upub::url!(ctx, "/actors/{id}/{follow___}"), Some(count as u64))
}

pub async fn page<const OUTGOING: bool>(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	use upub::model::relation::Column::{Follower, Following, FollowerInstance, FollowingInstance};
	let follow___ = if OUTGOING { "following" } else { "followers" };

	let (limit, offset) = page.pagination();

	let (user, config) = model::actor::Entity::find_by_ap_id(&ctx.uid(&id))
		.find_also_related(model::config::Entity)
		.one(ctx.db())
		.await?
		.ok_or_else(ApiError::not_found)?;

	let hidden = match config {
		// assume all remote users have private followers
		//  this because we get to see some of their "private" followers if they follow local users,
		//  and there is no mechanism to broadcast privacy on/off, so we could be leaking followers. to
		//  mitigate this, just assume them all private: local users can only see themselves and remote
		//  fetchers can only see relations from their instance (meaning likely zero because we only
		//  store relations for which at least one end is on local instance)
		None => true,
		Some(config) => {
			if OUTGOING { !config.show_followers } else { !config.show_following }
		}
	};

	let mut filter = Condition::all()
		.add(model::relation::Column::Accept.is_not_null())
		.add(if OUTGOING { Follower } else { Following }.eq(user.internal));

	if hidden {
		match auth {
			Identity::Anonymous => return Err(ApiError::unauthorized()),
			Identity::Local { id, internal } => {
				if id != ctx.uid(&id) {
					filter = filter.add(if OUTGOING { Following } else { Follower }.eq(internal));
				}
			},
			Identity::Remote { internal, .. } => {
				filter = filter.add(if OUTGOING { FollowingInstance } else { FollowerInstance }.eq(internal));
			},
		}
	}

	let join = if OUTGOING {
		model::relation::Relation::ActorsFollowing.def()
	} else {
		model::relation::Relation::ActorsFollower.def()
	};

	let following = model::relation::Entity::find()
		.filter(filter)
		.join(sea_orm::JoinType::LeftJoin, join)
		.select_only()
		.select_column(model::actor::Column::Id)
		.limit(limit)
		.offset(page.offset.unwrap_or(0))
		.into_tuple::<String>()
		.all(ctx.db())
		.await?;

	crate::builders::collection_page(
		&upub::url!(ctx, "/actors/{id}/{follow___}/page"),
		offset, limit,
		apb::Node::links(following),
	)
}
