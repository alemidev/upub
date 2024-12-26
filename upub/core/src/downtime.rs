use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter};


pub async fn get(db: &impl ConnectionTrait, domain: &str) -> Result<Option<chrono::DateTime<chrono::Utc>>, DbErr> {
	Ok(
		crate::model::downtime::Entity::find()
			.filter(crate::model::downtime::Column::Domain.eq(domain))
			.one(db)
			.await?
			.map(|x| x.published)
	)
}

pub async fn set(db: &impl ConnectionTrait, domain: &str) -> Result<(), DbErr> {
	match crate::model::downtime::Entity::find()
		.filter(crate::model::downtime::Column::Domain.eq(domain))
		.one(db)
		.await?
	{
		Some(model) => {
			let mut active = model.into_active_model();
			active.published = sea_orm::ActiveValue::Set(chrono::Utc::now());
			active.update(db).await?;
		},
		None => {
			crate::model::downtime::ActiveModel {
				internal: sea_orm::ActiveValue::NotSet,
				domain: sea_orm::ActiveValue::Set(domain.to_string()),
				published: sea_orm::ActiveValue::Set(chrono::Utc::now()),
			}
				.insert(db)
				.await?;
		},
	}

	Ok(())
}

pub async fn unset(db: &impl ConnectionTrait, domain: &str) -> Result<(), DbErr> {
	crate::model::downtime::Entity::delete_many()
		.filter(crate::model::downtime::Column::Domain.eq(domain))
		.exec(db)
		.await?;
	Ok(())
}
