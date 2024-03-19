use sea_orm::entity::prelude::*;

use crate::activitystream::{self, types::ActorType};

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

impl activitystream::Object for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn full_type(&self) -> Option<activitystream::BaseType> {
		Some(activitystream::BaseType::Object(activitystream::ObjectType::Actor(self.actor_type)))
	}

	fn name (&self) -> Option<&str> {
		Some(&self.name)
	}
}

impl Model {
	pub fn new(object: &impl activitystream::Object) -> Result<Self, super::FieldError> {
		let Some(activitystream::BaseType::Object(activitystream::ObjectType::Actor(t))) = object.full_type() else {
			return Err(super::FieldError("type")); // TODO maybe just wrong? better errors!
		};
		Ok(Model {
			id: object.id().ok_or(super::FieldError("id"))?.to_string(),
			actor_type: t,
			name: object.name().ok_or(super::FieldError("name"))?.to_string(),
		})
	}
}
