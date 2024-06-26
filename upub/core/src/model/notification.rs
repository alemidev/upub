use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "notifications")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub activity: i64,
	pub actor: i64,
	pub seen: bool,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Activity",
		to = "super::activity::Column::Internal",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Activities,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Actor",
		to = "super::actor::Column::Internal",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Actors,
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Activities.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
