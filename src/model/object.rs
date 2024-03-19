use sea_orm::entity::prelude::*;

use crate::activitystream::{Object, types::{BaseType, ObjectType, StatusType}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "objects")]
pub struct Model {
	#[sea_orm(primary_key)]
	/// must be full uri!!! maybe not great?
	pub id: String,
	pub object_type: StatusType,
	pub attributed_to: Option<String>,
	pub name: Option<String>,
	pub summary: Option<String>,
	pub content: Option<String>,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl crate::activitystream::Object for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn full_type(&self) -> Option<crate::activitystream::BaseType> {
		Some(BaseType::Object(ObjectType::Status(self.object_type)))
	}

	fn attributed_to (&self) -> Option<&str> {
		self.attributed_to.as_deref()
	}

	fn name (&self) -> Option<&str> {
		self.name.as_deref()
	}

	fn summary (&self) -> Option<&str> {
		self.summary.as_deref()
	}

	fn content(&self) -> Option<&str> {
		self.content.as_deref()
	}

	fn published (&self) -> Option<chrono::DateTime<chrono::Utc>> {
		Some(self.published)
	}
}

impl Model {
	pub fn new(object: &impl Object) -> Result<Self, super::FieldError> {
		let Some(BaseType::Object(ObjectType::Status(t))) = object.full_type() else {
			return Err(super::FieldError("type")); // TODO maybe just wrong? better errors!
		};
		Ok(Model {
			id: object.id().ok_or(super::FieldError("id"))?.to_string(),
			object_type: t,
			attributed_to: object.attributed_to().map(|x| x.to_string()),
			name: object.name().map(|x| x.to_string()),
			summary: object.summary().map(|x| x.to_string()),
			content: object.content().map(|x| x.to_string()),
			published: object.published().ok_or(super::FieldError("published"))?,
		})
	}
}
