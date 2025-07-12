use apb::ObjectMut;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "question_options")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub object: i64,
	pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::question_answer::Entity")]
	QuestionAnswers,
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Questions,
}

impl Related<super::question_answer::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::QuestionAnswers.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Questions.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl crate::ext::IntoActivityPub for Model {
	fn into_activity_pub_json(self, _ctx: &crate::Context) -> serde_json::Value {
		apb::new()
			.set_name(Some(self.name))
			// .set_object_type(Some(apb::ObjectType::Note))
	}
}
