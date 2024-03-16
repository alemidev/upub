
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

	fn full_type(&self) -> Option<activitystream::Type> {
		Some(activitystream::Type::ActorType(self.actor_type))
	}

	fn name (&self) -> Option<&str> {
		Some(&self.name)
	}
}
