use sea_orm::entity::prelude::*;

use crate::activitystream::{node::InsertStr, object::{Actor, Object, ObjectType}, Base, BaseType, Node};

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

impl Base for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Object(self.object_type))
	}
}

impl Object for Model {
	fn object_type(&self) -> Option<ObjectType> {
		Some(self.object_type)
	}

	fn attributed_to(&self) -> Node<impl Actor> {
		Node::<serde_json::Value>::from(self.attributed_to.as_deref())
	}

	fn name(&self) -> Option<&str> {
		self.name.as_deref()
	}

	fn summary(&self) -> Option<&str> {
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
		Ok(Model {
			id: object.id().ok_or(super::FieldError("id"))?.to_string(),
			object_type: object.object_type().ok_or(super::FieldError("type"))?,
			attributed_to: object.attributed_to().id().map(|x| x.to_string()),
			name: object.name().map(|x| x.to_string()),
			summary: object.summary().map(|x| x.to_string()),
			content: object.content().map(|x| x.to_string()),
			published: object.published().ok_or(super::FieldError("published"))?,
		})
	}
}

impl super::ToJson for Model {
	fn json(&self) -> serde_json::Value {
		let mut map = serde_json::Map::new();
		map.insert_str("id", Some(&self.id));
		map.insert_str("type", Some(self.object_type.as_ref()));
		map.insert_str("attributedTo", self.attributed_to.as_deref());
		map.insert_str("name", self.name.as_deref());
		map.insert_str("summary", self.summary.as_deref());
		map.insert_str("content", self.content.as_deref());
		map.insert_timestr("published", Some(self.published));
		serde_json::Value::Object(map)
	}
}
