use sea_orm::EntityTrait;

pub async fn fix(ctx: upub::Context, likes: bool, shares: bool, replies: bool) -> Result<(), sea_orm::DbErr> {
	use futures::TryStreamExt;
	let db = ctx.db();

	if likes {
		tracing::info!("fixing likes...");
		let mut store = std::collections::HashMap::new();
		{
			let mut stream = upub::model::like::Entity::find().stream(db).await?;
			while let Some(like) = stream.try_next().await? {
				store.insert(like.object, store.get(&like.object).unwrap_or(&0) + 1);
			}
		}

		for (k, v) in store {
			let m = upub::model::object::ActiveModel {
				internal: sea_orm::Set(k),
				likes: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = upub::model::object::Entity::update(m)
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
			let mut stream = upub::model::announce::Entity::find().stream(db).await?;
			while let Some(share) = stream.try_next().await? {
				store.insert(share.object, store.get(&share.object).unwrap_or(&0) + 1);
			}
		}

		for (k, v) in store {
			let m = upub::model::object::ActiveModel {
				internal: sea_orm::Set(k),
				announces: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = upub::model::object::Entity::update(m)
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
				id: sea_orm::Set(k.clone()),
				replies: sea_orm::Set(v),
				..Default::default()
			};
			if let Err(e) = upub::model::object::Entity::update(m)
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
