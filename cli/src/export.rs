use futures::TryStreamExt;
use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};
use upub::selector::{RichFillable, RichObject};


pub async fn export(
	ctx: upub::Context,
	actor: String,
	file: std::path::PathBuf,
	pretty: bool,
) -> Result<(), Box<dyn std::error::Error>> {
	let uid = ctx.uid(&actor);
	let internal = upub::model::actor::Entity::ap_to_internal(&uid, ctx.db())
		.await?
		.ok_or(sea_orm::DbErr::RecordNotFound(format!("could not find user {actor}")))?;

	let mut objects = Vec::new();
	
	let mut stream = upub::Query::feed(upub::query_feed_opts!(Some(internal), true))
		.filter(upub::model::object::Column::AttributedTo.eq(uid))
		.order_by_asc(upub::model::addressing::Column::Published)
		.order_by_asc(upub::model::activity::Column::Internal)
		.into_model::<RichObject>()
		.stream(ctx.db())
		.await?;

	while let Some(obj) = stream.try_next().await? {
		objects.push(ctx.ap(obj.load_batched_models(ctx.db()).await?));
	}

	let writer = std::fs::File::create(file)?;

	if pretty {
		serde_json::to_writer_pretty(writer, &objects)?;
	} else {
		serde_json::to_writer(writer, &objects)?;
	}

	Ok(())
}
