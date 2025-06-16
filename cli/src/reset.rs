use sea_orm::{EntityTrait, IntoActiveModel, QueryFilter, ColumnTrait, ActiveModelTrait};

pub async fn reset(
	ctx: upub::Context,
	username: String,
	password: String,
) -> Result<(), sea_orm::DbErr> {

	let mut user = upub::model::credential::Entity::find()
		.filter(upub::model::credential::Column::Login.eq(&username))
		.one(ctx.db())
		.await?
		.ok_or(sea_orm::DbErr::RecordNotFound("no such user".to_string()))?
		.into_active_model();

	user.password = sea_orm::Set(sha256::digest(password));

	user.update(ctx.db()).await?;

	tracing::info!("reset password of user: {username}");
	
	Ok(())
}
