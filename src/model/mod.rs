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
		actor_type: sea_orm::Set(super::activitystream::types::ActorType::Person),
	}).exec(db).await?;

	object::Entity::insert(object::ActiveModel {
		id: sea_orm::Set("http://localhost:3000/objects/4e28d30b-33c1-4336-918b-6fbe592bdd44".into()),
		name: sea_orm::Set(None),
		object_type: sea_orm::Set(crate::activitystream::types::StatusType::Note),
		attributed_to: sea_orm::Set(Some("http://localhost:3000/users/root".into())),
		summary: sea_orm::Set(None),
		content: sea_orm::Set(Some("Hello world!".into())),
		published: sea_orm::Set(chrono::Utc::now()),
	}).exec(db).await?;

	activity::Entity::insert(activity::ActiveModel {
		id: sea_orm::Set("http://localhost:3000/activities/ebac57e1-9828-438c-be34-a44a52de7641".into()),
		activity_type: sea_orm::Set(crate::activitystream::types::ActivityType::Create),
		actor: sea_orm::Set("http://localhost:3000/users/root".into()),
		object: sea_orm::Set(Some("http://localhost:3000/obkects/4e28d30b-33c1-4336-918b-6fbe592bdd44".into())),
		target: sea_orm::Set(None),
		published: sea_orm::Set(chrono::Utc::now()),
	}).exec(db).await?;

	Ok(())
}
