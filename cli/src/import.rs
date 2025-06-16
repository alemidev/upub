use apb::{Activity, ActivityMut, Base, BaseMut, CollectionMut, Document, DocumentMut, Object, ObjectMut};
use sea_orm::TransactionTrait;


pub async fn import(
	ctx: upub::Context,
	file: std::path::PathBuf,
	from: String,
	to: String,
	attachment_base: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
	// TODO worth including tokio/fs to do this async? it's a CLI task anyway
	let raw_content = std::fs::read_to_string(file)?;
	let objects : Vec<serde_json::Value> = serde_json::from_str(&raw_content)?;

	let tx = ctx.db().begin().await?;

	for mut obj in objects {
		if let Some(data) = obj.get_mut("data") {
			obj = data.take();
		}

		let Ok(oid) = obj.id() else {
			tracing::warn!("skipping object without id : {obj}");
			continue;
		};

		let attributed_to = match obj.attributed_to().id() {
			Ok(id) => id,
			Err(_) => match obj.actor().id() {
				Ok(id) => id,
				Err(_) => {
					tracing::warn!("skipping object without author: {obj}");
					continue;
				},
			},
		};

		if attributed_to != from {
			tracing::warn!("skipping object not belonging to requested user: {obj}");
			continue;
		}

		let normalized_attachments = match attachment_base {
			Some(ref attachment_base) => {
				let mut out = Vec::new();
				for attachment in obj.attachment().flat() {
					let Ok(doc) = attachment.inner() else {
						tracing::warn!("skipping non embedded attachment: {attachment:?}");
						continue;
					};
					out.push(
						apb::new()
							.set_document_type(doc.document_type().ok())
							.set_name(doc.name().ok())
							.set_media_type(doc.media_type().ok())
							.set_url(apb::Node::link(
								format!("{attachment_base}/{}", doc.url().id().unwrap_or_default().split('/').next_back().unwrap_or_default())
							))
					);
				}
				apb::Node::array(out)
			},
			None => obj.attachment()
		};

		let normalized_summary = obj.summary()
			.ok()
			.filter(|x| !x.is_empty());

		let announces_count = match obj.get("announcement_count") {
			Some(v) => v.as_u64().unwrap_or_default(),
			None => obj.shares().flat().len() as u64,
		};

		let replies_count = match obj.get("repliesCount") {
			Some(v) => v.as_u64().unwrap_or_default(),
			None => obj.replies().flat().len() as u64,
		};

		let likes_count = match obj.get("like_count") {
			Some(v) => v.as_u64().unwrap_or_default(),
			None => obj.likes().flat().len() as u64,
		};

		let normalized_object = obj
			.set_id(Some(ctx.oid(&upub::Context::new_id())))
			.set_attributed_to(apb::Node::link(to.clone()))
			.set_summary(normalized_summary)
			.set_shares(apb::Node::object(
				apb::new()
					.set_total_items(Some(announces_count))
			))
			.set_likes(apb::Node::object(
				apb::new()
					.set_total_items(Some(likes_count))
			))
			.set_replies(apb::Node::object(
				apb::new()
						.set_total_items(Some(replies_count))
			))
			.set_attachment(normalized_attachments);

		let activity = apb::new()
			.set_id(Some(ctx.aid(&upub::Context::new_id())))
			.set_activity_type(Some(apb::ActivityType::Create))
			.set_actor(apb::Node::link(to.clone()))
			.set_published(normalized_object.published().ok())
			.set_object(apb::Node::object(normalized_object));

		if let Err(e) = upub::traits::process::process_create(&ctx, activity, &tx).await {
			tracing::error!("could not insert object {oid}: {e} ({e:?})");
		}
	}

	tx.commit().await?;

	Ok(())
}
