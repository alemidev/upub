use futures::TryStreamExt;
use sea_orm::{ActiveValue::{Set, NotSet}, ColumnTrait, EntityTrait, QueryFilter};

use crate::server::fetcher::Fetcher;

pub async fn update_users(ctx: crate::server::Context, days: i64) -> crate::Result<()> {
	let mut count = 0;
	let mut insertions = Vec::new();

	{
		let mut stream = crate::model::actor::Entity::find()
			.filter(crate::model::actor::Column::Updated.lt(chrono::Utc::now() - chrono::Duration::days(days)))
			.stream(ctx.db())
			.await?;


		while let Some(user) = stream.try_next().await? {
			if ctx.is_local(&user.id) { continue }
			match ctx.pull_user(&user.id).await {
				Err(e) => tracing::warn!("could not update user {}: {e}", user.id),
				Ok(doc) => match crate::model::actor::ActiveModel::new(&doc) {
					Ok(mut u) => {
						u.internal = NotSet;
						u.updated = Set(chrono::Utc::now());
						let uid = u.id.take().unwrap_or(user.id.clone());
						insertions.push((uid, u));
						count += 1;
					},
					Err(e) => tracing::warn!("failed deserializing user '{}': {e}", user.id),
				},
			}
		}
	}

	for (uid, user_model) in insertions {
		tracing::info!("updating user {}", uid);
		crate::model::actor::Entity::update(user_model)
			.filter(crate::model::actor::Column::Id.eq(uid))
			.exec(ctx.db())
			.await?;
	}

	tracing::info!("updated {count} users");

	Ok(())
}
