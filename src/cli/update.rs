use futures::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::server::{fetcher::Fetcher, Context};

pub async fn update_users(db: sea_orm::DatabaseConnection, domain: String, days: i64) -> crate::Result<()> {
	let ctx = Context::new(db, domain).await?;
	let mut count = 0;
	let mut insertions = Vec::new();

	{
		let mut stream = crate::model::user::Entity::find()
			.filter(crate::model::user::Column::Updated.lt(chrono::Utc::now() - chrono::Duration::days(days)))
			.stream(ctx.db())
			.await?;


		while let Some(user) = stream.try_next().await? {
			match ctx.pull_user(&user.id).await {
				Err(e) => tracing::warn!("could not update user {}: {e}", user.id),
				Ok(u) => {
					insertions.push(u);
					count += 1;
				},
			}
		}
	}

	for u in insertions {
		tracing::info!("updating user {}", u.id);
		crate::model::user::Entity::delete_by_id(&u.id).exec(ctx.db()).await?;
		crate::model::user::Entity::insert(u.into_active_model()).exec(ctx.db()).await?;
	}

	tracing::info!("updated {count} users");

	Ok(())
}
