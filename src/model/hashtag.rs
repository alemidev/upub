use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "hashtags")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub object: i32,
	pub name: String,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Objects,
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
