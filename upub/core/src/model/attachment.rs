use apb::{DocumentMut, DocumentType, ObjectMut};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "attachments")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub url: String,
	pub object: i64,
	pub document_type: DocumentType,
	pub name: Option<String>,
	pub media_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Internal",
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

impl Model {
	pub fn ap(self) -> serde_json::Value {
		apb::new()
			.set_url(apb::Node::link(self.url))
			.set_document_type(Some(self.document_type))
			.set_media_type(Some(&self.media_type))
			.set_name(self.name.as_deref())
	}
}
