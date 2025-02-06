use apb::{ActivityMut, Base, BaseMut, Object, ObjectMut};
use sea_orm::TransactionTrait;


pub async fn import(ctx: upub::Context, file: std::path::PathBuf, from: String, to: String) -> Result<(), Box<dyn std::error::Error>> {
	// TODO worth including tokio/fs to do this async? it's a CLI task anyway
	let raw_content = std::fs::read_to_string(file)?;
	let objects : Vec<serde_json::Value> = serde_json::from_str(&raw_content)?;

	let tx = ctx.db().begin().await?;

	for obj in objects {
		let Ok(oid) = obj.id() else {
			tracing::warn!("skipping object without id : {obj}");
			continue;
		};
		let Ok(attributed_to) = obj.attributed_to().id() else {
			tracing::warn!("skipping object without author: {obj}");
			continue;
		};
		if attributed_to != from {
			tracing::warn!("skipping object not belonging to requested user: {obj}");
			continue;
		}

		let activity = apb::new()
			.set_id(Some(upub::Context::new_id()))
			.set_activity_type(Some(apb::ActivityType::Create))
			.set_actor(apb::Node::link(to.clone()))
			.set_published(obj.published().ok())
			.set_object(apb::Node::object(obj.set_attributed_to(apb::Node::link(to.clone()))));
		if let Err(e) = upub::traits::process::process_create(&ctx, activity, &tx).await {
			tracing::error!("could not insert object {oid}: {e} ({e:?})");
		}
	}

	tx.commit().await?;

	Ok(())
}
