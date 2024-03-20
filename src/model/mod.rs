pub mod user;
pub mod object;
pub mod activity;

#[derive(Debug, Clone, thiserror::Error)]
#[error("missing required field: '{0}'")]
pub struct FieldError(pub &'static str);

pub async fn faker(db: &sea_orm::DatabaseConnection, domain: String) -> Result<(), sea_orm::DbErr> {
	use sea_orm::EntityTrait;

	user::Entity::insert(user::ActiveModel {
		id: sea_orm::Set(format!("{domain}/users/root")),
		name: sea_orm::Set("root".into()),
		actor_type: sea_orm::Set(super::activitystream::object::actor::ActorType::Person),
	}).exec(db).await?;

	for i in (0..100).rev() {
		let oid = uuid::Uuid::new_v4();
		let aid = uuid::Uuid::new_v4();
		object::Entity::insert(object::ActiveModel {
			id: sea_orm::Set(format!("{domain}/objects/{oid}")),
			name: sea_orm::Set(None),
			object_type: sea_orm::Set(crate::activitystream::object::ObjectType::Note),
			attributed_to: sea_orm::Set(Some(format!("{domain}/users/root"))),
			summary: sea_orm::Set(None),
			content: sea_orm::Set(Some(format!("[{i}] Tic(k). Quasiparticle of intensive multiplicity. Tics (or ticks) are intrinsically several components of autonomously numbering anorganic populations, propagating by contagion between segmentary divisions in the order of nature. Ticks - as nonqualitative differentially-decomposable counting marks - each designate a multitude comprehended as a singular variation in tic(k)-density."))),
			published: sea_orm::Set(chrono::Utc::now() - std::time::Duration::from_secs(60*i)),
		}).exec(db).await?;

		activity::Entity::insert(activity::ActiveModel {
			id: sea_orm::Set(format!("{domain}/activities/{aid}")),
			activity_type: sea_orm::Set(crate::activitystream::object::activity::ActivityType::Create),
			actor: sea_orm::Set(format!("{domain}/users/root")),
			object: sea_orm::Set(Some(format!("{domain}/objects/{oid}"))),
			target: sea_orm::Set(None),
			published: sea_orm::Set(chrono::Utc::now() - std::time::Duration::from_secs(60*i)),
		}).exec(db).await?;
	}

	Ok(())
}
