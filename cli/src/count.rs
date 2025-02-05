use sea_orm::{ActiveModelTrait, EntityTrait};

pub async fn count(ctx: upub::Context, likes: bool, shares: bool, replies: bool) -> Result<(), sea_orm::DbErr> {
	use futures::TryStreamExt;
	let db = ctx.db();

	if likes {
		tracing::info!("counting likes...");
		let mut store = std::collections::HashMap::new();
		{
			let mut stream = upub::model::like::Entity::find().stream(db).await?;
			while let Some(like) = stream.try_next().await? {
				store.insert(like.object, store.get(&like.object).unwrap_or(&0) + 1);
			}
		}

		for (k, v) in store {
			let m = upub::model::object::ActiveModel {
				internal: sea_orm::Unchanged(k),
				likes: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = m.update(db).await {
				tracing::warn!("record not updated ({k}): {e}");
			}
		}
	}

	if shares {
		tracing::info!("counting shares...");
		let mut store = std::collections::HashMap::new();
		{
			let mut stream = upub::model::announce::Entity::find().stream(db).await?;
			while let Some(share) = stream.try_next().await? {
				store.insert(share.object, store.get(&share.object).unwrap_or(&0) + 1);
			}
		}

		for (k, v) in store {
			let m = upub::model::object::ActiveModel {
				internal: sea_orm::Unchanged(k),
				announces: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = m.update(db).await {
				tracing::warn!("record not updated ({k}): {e}");
			}
		}
	}

	if replies {
		tracing::info!("counting replies...");
		let mut store = std::collections::HashMap::new();
		{
			let mut stream = upub::model::object::Entity::find().stream(db).await?;
			while let Some(object) = stream.try_next().await? {
				if let Some(reply) = object.in_reply_to {
					let before = store.get(&reply).unwrap_or(&0);
					store.insert(reply, before + 1);
				}
			}
		}

		for (k, v) in store {
			let m = upub::model::object::ActiveModel {
				id: sea_orm::Unchanged(k.clone()),
				replies: sea_orm::Set(v),
				..Default::default()
			};
			// TODO will update work with non-primary-key field??
			if let Err(e) = m.update(db).await {
				tracing::warn!("record not updated ({k}): {e}");
			}
		}
	}

	tracing::info!("done running fix tasks");
	Ok(())
}
