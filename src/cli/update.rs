use futures::TryStreamExt;
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

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
						u.internal = Set(user.internal);
						u.updated = Set(chrono::Utc::now());
						insertions.push((user.id, u));
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
			.exec(ctx.db())
			.await?;
	}

	tracing::info!("updated {count} users");

	Ok(())
}
