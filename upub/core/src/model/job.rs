use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum JobType {
	Inbound = 1,
	Outbound = 2,
	Local = 3,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub job_type: JobType,
	pub actor: String,
	pub target: Option<String>,
	#[sea_orm(unique)]
	pub activity: String,
	pub payload: Option<String>,
	pub published: ChronoDateTimeUtc,
	pub not_before: ChronoDateTimeUtc,
	pub attempt: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
	pub fn next_attempt(&self) -> ChronoDateTimeUtc {
		match self.attempt {
			0 => chrono::Utc::now() + std::time::Duration::from_secs(10),
			1 => chrono::Utc::now() + std::time::Duration::from_secs(60),
			2 => chrono::Utc::now() + std::time::Duration::from_secs(5 * 60),
			3 => chrono::Utc::now() + std::time::Duration::from_secs(20 * 60),
			4 => chrono::Utc::now() + std::time::Duration::from_secs(60 * 60),
			5 => chrono::Utc::now() + std::time::Duration::from_secs(12 * 60 * 60),
			_ => chrono::Utc::now() + std::time::Duration::from_secs(24 * 60 * 60),
		}
	}

	pub fn expired(&self) -> bool {
		chrono::Utc::now() - self.published > chrono::Duration::days(7)
	}

	pub fn repeat(self) -> ActiveModel {
		ActiveModel {
			internal: sea_orm::ActiveValue::NotSet,
			job_type: sea_orm::ActiveValue::Set(self.job_type),
			not_before: sea_orm::ActiveValue::Set(self.next_attempt()),
			actor: sea_orm::ActiveValue::Set(self.actor),
			target: sea_orm::ActiveValue::Set(self.target),
			payload: sea_orm::ActiveValue::Set(self.payload),
			activity: sea_orm::ActiveValue::Set(self.activity),
			published: sea_orm::ActiveValue::Set(self.published),
			attempt: sea_orm::ActiveValue::Set(self.attempt + 1),
		}
	}
}
