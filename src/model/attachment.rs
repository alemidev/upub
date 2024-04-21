use apb::{DocumentMut, ObjectMut};
use sea_orm::{entity::prelude::*, Set};

use crate::routes::activitypub::jsonld::LD;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "attachments")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i64,

	pub url: String,
	pub object: String,
	pub document_type: apb::DocumentType,
	pub name: Option<String>,
	pub media_type: String,
	pub created: ChronoDateTimeUtc,
}

impl ActiveModel {
	pub fn new(document: &impl apb::Document, object: String) -> Result<ActiveModel, super::FieldError> {
		Ok(ActiveModel {
			id: sea_orm::ActiveValue::NotSet,
			object: Set(object),
			url: Set(document.url().id().ok_or(super::FieldError("url"))?),
			document_type: Set(document.document_type().ok_or(super::FieldError("type"))?),
			media_type: Set(document.media_type().ok_or(super::FieldError("mediaType"))?.to_string()),
			name: Set(document.name().map(|x| x.to_string())),
			created: Set(document.published().unwrap_or(chrono::Utc::now())),
		})
	}
}

impl Model {
	pub fn ap(self) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_url(apb::Node::link(self.url))
			.set_document_type(Some(self.document_type))
			.set_media_type(Some(&self.media_type))
			.set_name(self.name.as_deref())
			.set_published(Some(self.created))
	}
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Id"
	)]
	Object,
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Object.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
