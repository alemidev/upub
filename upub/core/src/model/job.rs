use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum JobType {
	Inbound = 1,
	Outbound = 2,
	Delivery = 3,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub job_type: JobType,
	pub actor: String,
	pub target: Option<String>,
	pub activity: String,
	pub payload: Option<serde_json::Value>,
	pub published: ChronoDateTimeUtc,
	pub not_before: ChronoDateTimeUtc,
	pub attempt: i16,
	pub error: Option<String>,
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

	pub fn repeat(self, error: Option<String>) -> ActiveModel {
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
			error: sea_orm::ActiveValue::Set(error),
		}
	}
}
