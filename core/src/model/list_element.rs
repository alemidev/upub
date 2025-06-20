use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "list_elements")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub list: i64,
	pub object: Option<i64>,
	pub actor: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Actor",
		to = "super::actor::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Actors,
	#[sea_orm(
		belongs_to = "super::list::Entity",
		from = "Column::List",
		to = "super::list::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Lists,
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Objects,
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl Related<super::list::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Lists.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
