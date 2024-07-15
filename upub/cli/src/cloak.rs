use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ActiveValue::{Set, Unchanged}, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns, TransactionTrait};
use upub::traits::{fetch::RequestError, Cloaker};

pub async fn cloak(ctx: upub::Context, post_contents: bool) -> Result<(), RequestError> {
	{
		let mut stream = upub::model::attachment::Entity::find()
			.filter(upub::model::attachment::Column::Url.not_like(format!("{}%", ctx.base())))
			.stream(ctx.db())
			.await?;

		while let Some(attachment) = stream.try_next().await? {
			tracing::info!("cloaking {}", attachment.url);
			let (sig, url) = ctx.cloak(&attachment.url);
			let mut model = attachment.into_active_model();
			model.url = Set(upub::url!(ctx, "/proxy/{sig}/{url}"));
			model.update(ctx.db()).await?;
		}
	}

	if post_contents {
		let mut stream = upub::model::object::Entity::find()
			.filter(upub::model::object::Column::Content.like("%<img%"))
			.select_only()
			.select_column(upub::model::object::Column::Internal)
			.select_column(upub::model::object::Column::Content)
			.into_tuple::<(i64, String)>()
			.stream(ctx.db())
			.await?;

		while let Some((internal, content)) = stream.try_next().await? {
			let sanitized = ctx.sanitize(&content);
			if sanitized != content {
				tracing::info!("sanitizing object #{internal}");
				let model = upub::model::object::ActiveModel {
					internal: Unchanged(internal),
					content: Set(Some(sanitized)),
					..Default::default()
				};
				model.update(ctx.db()).await?;
			}
		}
	}

	Ok(())
}
