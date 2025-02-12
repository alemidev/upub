use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait};

pub async fn fix_attachments_types(ctx: upub::Context) -> Result<(), Box<dyn std::error::Error>> {
	tracing::info!("fixing attachments documentType...");
	let tx = ctx.db().begin().await?;

	let mut stream = upub::model::attachment::Entity::find()
		.filter(upub::model::attachment::Column::DocumentType.eq(apb::DocumentType::Document))
		.stream(ctx.db())
		.await?;

	while let Some(attachment) = stream.try_next().await? {
		let Some((mime_kind, _mime_type)) = attachment.media_type.split_once('/') else { continue };

		let document_type = match mime_kind {
			"image" => apb::DocumentType::Image,
			"video" => apb::DocumentType::Video,
			"audio" => apb::DocumentType::Audio,
			"text"  => apb::DocumentType::Page,
			_ => continue,
		};

		tracing::info!("updating {} to {document_type}", attachment.url);
		let mut active = attachment.into_active_model();
		active.document_type = Set(document_type);
		active.update(&tx).await?;
	}

	tracing::info!("committing transaction...");
	tx.commit().await?;
	tracing::info!("done");

	Ok(())
}
