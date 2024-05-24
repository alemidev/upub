use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "announces")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub actor: i32,
	pub announces: i32,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Actor",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Actors,
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Announces",
		to = "super::object::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Objects,
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
