use std::collections::HashSet;

use apb::{ActivityMut, BaseMut, ObjectMut};
use futures::TryStreamExt;
use sea_orm::{ActiveValue::{Set, NotSet}, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, SelectColumns};


pub async fn nuke(ctx: upub::Context, for_real: bool, delete_posts: bool) -> Result<(), sea_orm::DbErr> {
	if !for_real {
		tracing::warn!("THIS IS A DRY RUN! pass --for-real to actually nuke this instance");
	}

	let mut to_undo = Vec::new();

	// TODO rather expensive to find all local users with a LIKE query, should add an isLocal flag
	let local_users_vec = upub::model::actor::Entity::find()
		.filter(upub::model::actor::Column::Id.like(format!("{}%", ctx.base())))
		.select_only()
		.select_column(upub::model::actor::Column::Internal)
		.into_tuple::<i64>()
		.all(ctx.db())
		.await?;
	
	let local_users : HashSet<i64> = HashSet::from_iter(local_users_vec);

	{
		let mut stream = upub::model::relation::Entity::find().stream(ctx.db()).await?;
		while let Some(like) = stream.try_next().await? {
			if local_users.contains(&like.follower) {
				to_undo.push(like.activity);
			} else if local_users.contains(&like.following) {
				if let Some(accept) = like.accept {
					to_undo.push(accept);
				}
			}
		}
	}

	for internal in to_undo {
		let Some(activity) = upub::model::activity::Entity::find_by_id(internal)
			.one(ctx.db())
			.await?
		else {
			tracing::error!("could not load activity #{internal}");
			continue;
		};

		let Some(oid) = activity.object
		else {
			tracing::error!("can't undo activity without object");
			continue;
		};

		let aid = ctx.aid(&upub::Context::new_id());
		let undo_activity = apb::new()
			.set_id(Some(&aid))
			.set_activity_type(Some(apb::ActivityType::Undo))
			.set_actor(apb::Node::link(activity.actor.clone()))
			.set_object(apb::Node::link(oid))
			.set_published(Some(chrono::Utc::now()));


		let job = upub::model::job::ActiveModel {
			internal: NotSet,
			activity: Set(aid.clone()),
			job_type: Set(upub::model::job::JobType::Outbound),
			actor: Set(activity.actor),
			target: Set(None),
			published: Set(chrono::Utc::now()),
			not_before: Set(chrono::Utc::now()),
			attempt: Set(0),
			payload: Set(Some(undo_activity)),
		};

		tracing::debug!("undoing {}", activity.id);

		if for_real {
			upub::model::job::Entity::insert(job).exec(ctx.db()).await?;
		}
	}

	Ok(())
}
