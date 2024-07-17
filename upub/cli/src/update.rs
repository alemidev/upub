use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use upub::traits::Fetcher;

pub async fn update_users(ctx: upub::Context, days: i64, limit: Option<u64>) -> Result<(), sea_orm::DbErr> {
	let mut count = 0;
	let mut stream = upub::model::actor::Entity::find()
		.filter(upub::model::actor::Column::Updated.lt(chrono::Utc::now() - chrono::Duration::days(days)))
		.stream(ctx.db())
		.await?;


	while let Some(user) = stream.try_next().await? {
		if ctx.is_local(&user.id) { continue }
		if let Some(limit) = limit {
			if count >= limit { break }
		}
		match ctx.pull(&user.id).await.map(|x| x.actor()) {
			Err(e) => tracing::warn!("could not update user {}: {e}", user.id),
			Ok(Err(e)) => tracing::warn!("could not update user {}: {e}", user.id),
			Ok(Ok(doc)) => match upub::AP::actor_q(&doc, Some(user.internal)) {
				Ok(mut u) => {
					tracing::info!("updating user {}", user.id);
					u.updated = Set(chrono::Utc::now());
					u.update(ctx.db()).await?;
					count += 1;
				},
				Err(e) => tracing::warn!("failed deserializing user '{}': {e}", user.id),
			},
		}
	}

	tracing::info!("updated {count} users");

	Ok(())
}
