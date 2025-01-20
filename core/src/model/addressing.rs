use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "addressing")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub actor: Option<i64>,
	pub instance: Option<i64>,
	pub activity: Option<i64>,
	pub object: Option<i64>,
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
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::Instance",
		to = "super::instance::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Instances,
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Internal",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Objects,
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

impl Related<super::instance::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Instances.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
