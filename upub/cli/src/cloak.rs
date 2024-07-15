use futures::TryStreamExt;
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set, Unchanged}, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, SelectColumns};
use upub::traits::{fetch::RequestError, Cloaker};

pub async fn cloak(ctx: upub::Context, post_contents: bool, actors: bool) -> Result<(), RequestError> {
	let local_base = format!("{}%", ctx.base());
	{
		let mut stream = upub::model::attachment::Entity::find()
			.filter(upub::model::attachment::Column::Url.not_like(&local_base))
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

	if actors {
		let mut stream = upub::model::actor::Entity::find()
			.filter(
				Condition::any()
					.add(upub::model::actor::Column::Image.not_like(&local_base))
					.add(upub::model::actor::Column::Icon.not_like(&local_base))
			)
			.select_only()
			.select_column(upub::model::actor::Column::Internal)
			.select_column(upub::model::actor::Column::Image)
			.select_column(upub::model::actor::Column::Icon)
			.into_tuple::<(i64, Option<String>, Option<String>)>()
			.stream(ctx.db())
			.await?;

		while let Some((internal, image, icon)) = stream.try_next().await? {
			if image.is_none() && icon.is_none() { continue }
			// TODO can this if/else/else be made nicer??
			let image = if let Some(img) = image {
				if !img.starts_with(ctx.base()) {
					Set(Some(ctx.cloaked(&img)))
				} else {
					NotSet
				}
			} else {
				NotSet
			};
			let icon = if let Some(icn) = icon {
				if !icn.starts_with(ctx.base()) {
					Set(Some(ctx.cloaked(&icn)))
				} else {
					NotSet
				}
			} else {
				NotSet
			};
			let model = upub::model::actor::ActiveModel {
				internal: Unchanged(internal),
				image, icon,
				..Default::default()
			};
			model.update(ctx.db()).await?;
		}
	}

	Ok(())
}
