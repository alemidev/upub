use sea_orm::EntityTrait;


pub async fn fix(ctx: crate::server::Context, likes: bool, shares: bool, replies: bool) -> crate::Result<()> {
	use futures::TryStreamExt;
	let db = ctx.db();

	if likes {
		tracing::info!("fixing likes...");
		let mut store = std::collections::HashMap::new();
		{
			let mut stream = crate::model::like::Entity::find().stream(db).await?;
			while let Some(like) = stream.try_next().await? {
				store.insert(like.likes.clone(), store.get(&like.likes).unwrap_or(&0) + 1);
			}
		}

		for (k, v) in store {
			let m = crate::model::object::ActiveModel {
				id: sea_orm::Set(k.clone()),
				likes: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = crate::model::object::Entity::update(m)
				.exec(db)
				.await
			{
				tracing::warn!("record not updated ({k}): {e}");
			}
		}
	}

	if shares {
		tracing::info!("fixing shares...");
		let mut store = std::collections::HashMap::new();
		{
			let mut stream = crate::model::share::Entity::find().stream(db).await?;
			while let Some(share) = stream.try_next().await? {
				store.insert(share.shares.clone(), store.get(&share.shares).unwrap_or(&0) + 1);
			}
		}

		for (k, v) in store {
			let m = crate::model::object::ActiveModel {
				id: sea_orm::Set(k.clone()),
				shares: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = crate::model::object::Entity::update(m)
				.exec(db)
				.await
			{
				tracing::warn!("record not updated ({k}): {e}");
			}
		}
	}

	if replies {
		tracing::info!("fixing replies...");
		let mut store = std::collections::HashMap::new();
		{
			let mut stream = crate::model::object::Entity::find().stream(db).await?;
			while let Some(object) = stream.try_next().await? {
				if let Some(reply) = object.in_reply_to {
					let before = store.get(&reply).unwrap_or(&0);
					store.insert(reply, before + 1);
				}
			}
		}

		for (k, v) in store {
			let m = crate::model::object::ActiveModel {
				id: sea_orm::Set(k.clone()),
				comments: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = crate::model::object::Entity::update(m)
				.exec(db)
				.await
			{
				tracing::warn!("record not updated ({k}): {e}");
			}
		}
	}

	tracing::info!("done running fix tasks");
	Ok(())
}
