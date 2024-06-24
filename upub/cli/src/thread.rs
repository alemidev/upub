use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use upub::traits::{fetch::PullError, Fetcher};

pub async fn thread(ctx: upub::Context) -> Result<(), PullError> {
	use futures::TryStreamExt;
	let db = ctx.db();

	tracing::info!("fixing contexts...");
	let mut stream = upub::model::object::Entity::find()
		.filter(upub::model::object::Column::Context.is_null())
		.stream(db)
		.await?;

	while let Some(mut object) = stream.try_next().await? {
		match object.in_reply_to {
			None => object.context = Some(object.id.clone()),
			Some(ref in_reply_to) => {
				let reply = ctx.fetch_object(in_reply_to, ctx.db()).await?;
				if let Some(context) = reply.context {
					object.context = Some(context);
				} else {
					continue;
				}
			},
		}
		tracing::info!("updating context of {}", object.id);
		upub::model::object::Entity::update(object.into_active_model())
			.exec(ctx.db())
			.await?;
	}

	tracing::info!("done fixing contexts");
	Ok(())
}
