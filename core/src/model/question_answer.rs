use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "question_answers")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub object: i64,
	pub actor: i64,
	pub answer: i64,
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
		belongs_to = "super::question_option::Entity",
		from = "Column::Answer",
		to = "super::question_option::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	QuestionOptions,
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

impl Related<super::question_option::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::QuestionOptions.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
