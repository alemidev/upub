pub mod user;
pub mod object;
pub mod activity;

#[derive(Debug, Clone, thiserror::Error)]
#[error("missing required field: '{0}'")]
pub struct FieldError(pub &'static str);

pub async fn faker(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
	use sea_orm::EntityTrait;

	user::Entity::insert(user::ActiveModel {
		id: sea_orm::Set("http://localhost:3000/users/root".into()),
		name: sea_orm::Set("root".into()),
		actor_type: sea_orm::Set(super::activitystream::object::actor::ActorType::Person),
	}).exec(db).await?;

	for i in (0..100).rev() {
		let oid = uuid::Uuid::new_v4();
		let aid = uuid::Uuid::new_v4();
		object::Entity::insert(object::ActiveModel {
			id: sea_orm::Set(format!("http://localhost:3000/objects/{oid}")),
			name: sea_orm::Set(None),
			object_type: sea_orm::Set(crate::activitystream::object::ObjectType::Note),
			attributed_to: sea_orm::Set(Some("http://localhost:3000/users/root".into())),
			summary: sea_orm::Set(None),
			content: sea_orm::Set(Some(format!("Hello world! {i}"))),
			published: sea_orm::Set(chrono::Utc::now() - std::time::Duration::from_secs(60*i)),
		}).exec(db).await?;

		activity::Entity::insert(activity::ActiveModel {
			id: sea_orm::Set(format!("http://localhost:3000/activities/{aid}")),
			activity_type: sea_orm::Set(crate::activitystream::object::activity::ActivityType::Create),
			actor: sea_orm::Set("http://localhost:3000/users/root".into()),
			object: sea_orm::Set(Some(format!("http://localhost:3000/objects/{oid}"))),
			target: sea_orm::Set(None),
			published: sea_orm::Set(chrono::Utc::now() - std::time::Duration::from_secs(60*i)),
		}).exec(db).await?;
	}

	Ok(())
}
