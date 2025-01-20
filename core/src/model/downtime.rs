use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "downtimes")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub domain: String,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::Domain",
		to = "super::instance::Column::Domain",
		on_update = "Cascade",
	)]
	Instances,
}

impl Related<super::instance::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Instances.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
