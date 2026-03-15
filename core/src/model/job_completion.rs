use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "job_completions")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub job: i64,
	pub completed: ChronoDateTimeUtc,
	pub duration: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::job::Entity",
		from = "Column::Job",
		to = "super::job::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Jobs,
}

impl Related<super::job::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Jobs.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl super::job::Model {
	pub async fn track_completion(&self, db: &impl sea_orm::ConnectionTrait) -> Result<(), sea_orm::DbErr> {
		use sea_orm::ActiveModelTrait;
		let model = ActiveModel {
			internal: sea_orm::ActiveValue::NotSet,
			job: sea_orm::ActiveValue::Set(self.internal),
			completed: sea_orm::ActiveValue::Set(chrono::Utc::now()),
			duration: sea_orm::ActiveValue::Set((chrono::Utc::now() - self.published).num_milliseconds()),
		};

		model.insert(db).await?;

		Ok(())
	}
}
