use futures::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, ActiveModelTrait};

macro_rules! ok_or_continue {
	($x:expr) => {
		match $x {
			Some(x) => x,
			None => continue,
		}
	};
}

pub async fn fix_activities(ctx: upub::Context, likes: bool, announces: bool) -> Result<(), Box<dyn std::error::Error>> {
	if likes {
		tracing::info!("fixing like activities...");
		let mut stream = upub::model::activity::Entity::find()
			.filter(upub::model::activity::Column::ActivityType.eq(apb::ActivityType::Like))
			.filter(upub::model::activity::Column::Object.is_not_null())
			.stream(ctx.db())
			.await?;

		while let Some(activity) = stream.try_next().await? {
			let oid = ok_or_continue!(activity.object);
			let internal_oid = ok_or_continue!(upub::model::object::Entity::ap_to_internal(&oid, ctx.db()).await?);
			let uid = activity.actor;
			let internal_uid = ok_or_continue!(upub::model::actor::Entity::ap_to_internal(&uid, ctx.db()).await?);
			if let Some(like) = upub::model::like::Entity::find()
				.filter(upub::model::like::Column::Object.eq(internal_oid))
				.filter(upub::model::like::Column::Actor.eq(internal_uid))
				.filter(upub::model::like::Column::Published.eq(activity.published))
				.one(ctx.db())
				.await?
			{
				let mut active = like.into_active_model();
				active.activity = sea_orm::Set(Some(activity.internal));
				active.update(ctx.db()).await?;
			}
		}
	}

	if announces {
		tracing::info!("fixing announce activities...");
		let mut stream = upub::model::activity::Entity::find()
			.filter(upub::model::activity::Column::ActivityType.eq(apb::ActivityType::Announce))
			.filter(upub::model::activity::Column::Object.is_not_null())
			.stream(ctx.db())
			.await?;

		while let Some(activity) = stream.try_next().await? {
			let oid = ok_or_continue!(activity.object);
			let internal_oid = ok_or_continue!(upub::model::object::Entity::ap_to_internal(&oid, ctx.db()).await?);
			let uid = activity.actor;
			let internal_uid = ok_or_continue!(upub::model::actor::Entity::ap_to_internal(&uid, ctx.db()).await?);
			if let Some(like) = upub::model::announce::Entity::find()
				.filter(upub::model::announce::Column::Object.eq(internal_oid))
				.filter(upub::model::announce::Column::Actor.eq(internal_uid))
				.filter(upub::model::announce::Column::Published.eq(activity.published))
				.one(ctx.db())
				.await?
			{
				let mut active = like.into_active_model();
				active.activity = sea_orm::Set(Some(activity.internal));
				active.update(ctx.db()).await?;
			}
		}
	}

	tracing::info!("done");

	Ok(())
}
