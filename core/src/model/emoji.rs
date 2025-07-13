use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "emojis")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub name: String,
	pub domain: String,
	pub uri: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::Domain",
		to = "super::instance::Column::Domain",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Domains,
}

impl Related<super::instance::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Domains.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
