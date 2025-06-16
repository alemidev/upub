use futures::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};


pub async fn export(
	ctx: upub::Context,
	actor: String,
	file: std::path::PathBuf,
	pretty: bool,
) -> Result<(), Box<dyn std::error::Error>> {

	let uid = ctx.uid(&actor);
	let mut objects = Vec::new();
	
	let mut stream = upub::model::object::Entity::find()
		.filter(upub::model::object::Column::AttributedTo.eq(uid))
		.order_by_asc(upub::model::object::Column::Published)
		.stream(ctx.db())
		.await?;

	while let Some(obj) = stream.try_next().await? {
		objects.push(ctx.ap(obj));
	}

	let writer = std::fs::File::create(file)?;

	if pretty {
		serde_json::to_writer_pretty(writer, &objects)?;
	} else {
		serde_json::to_writer(writer, &objects)?;
	}

	Ok(())
}
