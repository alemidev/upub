use sea_orm::entity::prelude::*;

use crate::activitystream::types::ObjectType;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "objects")]
pub struct Model {
	#[sea_orm(primary_key)]
	/// must be full uri!!! maybe not great?
	pub id: String,
	pub object_type: ObjectType,
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

	fn full_type(&self) -> Option<crate::activitystream::Type> {
		Some(crate::activitystream::Type::ObjectType(self.object_type))
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
