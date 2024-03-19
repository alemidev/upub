use sea_orm::entity::prelude::*;

use crate::activitystream::{node::InsertStr, object::{Actor, ActorType}, Base, BaseType, Object, ObjectType};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
	#[sea_orm(primary_key)]
	/// must be user@instance.org, even if local! TODO bad design...
	pub id: String,

	pub actor_type: ActorType,

	pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Base for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Object(ObjectType::Actor(self.actor_type)))
	}
}

impl Object for Model {
	fn name (&self) -> Option<&str> {
		Some(&self.name)
	}
}

impl Actor for Model {
	fn actor_type(&self) -> Option<ActorType> {
		Some(self.actor_type)
	}
}

impl Model {
	pub fn new(object: &impl Actor) -> Result<Self, super::FieldError> {
		Ok(Model {
			id: object.id().ok_or(super::FieldError("id"))?.to_string(),
			actor_type: object.actor_type().ok_or(super::FieldError("type"))?,
			name: object.name().ok_or(super::FieldError("name"))?.to_string(),
		})
	}
}

impl super::ToJson for Model {
	fn json(&self) -> serde_json::Value {
		let mut map = serde_json::Map::new();
		map.insert_str("id", Some(&self.id));
		map.insert_str("type", Some(self.actor_type.as_ref()));
		map.insert_str("name", Some(&self.name));
		serde_json::Value::Object(map)
	}
}
