use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "deliveries")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub actor: i32,
	pub target: String,
	pub activity: i32,
	pub created: ChronoDateTimeUtc,
	pub not_before: ChronoDateTimeUtc,
	pub attempt: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Activity",
		to = "super::activity::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Activities,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Actor",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Actors,
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Activities.def()
	}
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
	pub fn next_delivery(&self) -> ChronoDateTimeUtc {
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
		chrono::Utc::now() - self.created > chrono::Duration::days(7)
	}
}
