use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ActiveValue::{Unchanged, Set}, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder};
use upub::traits::Fetcher;

pub async fn update_users(ctx: upub::Context, days: i64, limit: Option<u64>) -> Result<(), sea_orm::DbErr> {
	let mut count = 0;
	let mut stream = upub::model::actor::Entity::find()
		.filter(upub::model::actor::Column::Updated.lt(chrono::Utc::now() - chrono::Duration::days(days)))
		.order_by_asc(upub::model::actor::Column::Updated)
		.stream(ctx.db())
		.await?;


	while let Some(user) = stream.try_next().await? {
		if ctx.is_local(&user.id) { continue }
		if let Some(limit) = limit {
			if count >= limit { break }
		}
		let server = upub::Context::server(&user.id);
		if upub::downtime::get(ctx.db(), &server).await?.is_some() { continue }
		match ctx.pull(&user.id).await.and_then(|x| x.actor()) {
			Err(upub::traits::fetch::RequestError::Fetch(status, msg)) => {
				if status.as_u16() == 410 {
					tracing::info!("user {} has been deleted", user.id);
					user.delete(ctx.db()).await?;
				}
				else if status.as_u16() == 404 {
					tracing::info!("user {} does not exist anymore", user.id);
					user.delete(ctx.db()).await?;
				}
				else {
					upub::downtime::set(ctx.db(), &server).await?;
					tracing::warn!("could not fetch user {}: failed with status {status} -- {msg}", user.id);
				}
			},
			Err(e) => {
				upub::downtime::set(ctx.db(), &server).await?;
				tracing::warn!("could not fetch user {}: {e}", user.id)
			},
			Ok(doc) => match ctx.resolve_user(doc, ctx.db()).await {
				Err(e) => {
					upub::downtime::set(ctx.db(), &server).await?;
					tracing::warn!("failed deserializing user '{}': {e}", user.id)
				},
				Ok(mut u) => {
					tracing::info!("updating user {}", user.id);
					u.internal = Unchanged(user.internal);
					u.updated = Set(chrono::Utc::now());
					u.update(ctx.db()).await?;
					count += 1;
				},
			},
		}
	}

	tracing::info!("updated {count} users");

	Ok(())
}
