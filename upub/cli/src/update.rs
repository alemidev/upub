use futures::TryStreamExt;
use sea_orm::{ActiveValue::{Unchanged, Set}, ColumnTrait, EntityTrait, QueryFilter};
use upub::traits::Fetcher;

pub async fn update_users(ctx: upub::Context, days: i64) -> Result<(), sea_orm::DbErr> {
	let mut count = 0;
	let mut insertions = Vec::new();

	{
		let mut stream = upub::model::actor::Entity::find()
			.filter(upub::model::actor::Column::Updated.lt(chrono::Utc::now() - chrono::Duration::days(days)))
			.stream(ctx.db())
			.await?;


		while let Some(user) = stream.try_next().await? {
			if ctx.is_local(&user.id) { continue }
			match ctx.pull(&user.id).await.map(|x| x.actor()) {
				Err(e) => tracing::warn!("could not update user {}: {e}", user.id),
				Ok(Err(e)) => tracing::warn!("could not update user {}: {e}", user.id),
				Ok(Ok(doc)) => match upub::AP::actor_q(&doc) {
					Ok(mut u) => {
						u.internal = Unchanged(user.internal);
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
		upub::model::actor::Entity::update(user_model)
			.exec(ctx.db())
			.await?;
	}

	tracing::info!("updated {count} users");

	Ok(())
}
