use sea_orm::entity::prelude::*;

use crate::activitystream;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "activities")]
pub struct Model {
	#[sea_orm(primary_key)]
	/// must be https://instance.org/users/:user , even if local! TODO bad design...
	pub id: String,

	pub activity_type: activitystream::types::ActivityType,
	pub actor: String, // TODO relates to USER
	pub object: Option<String>, // TODO relates to NOTES maybe????? maybe other tables??????
	pub target: Option<String>, // TODO relates to USER maybe??
	pub published: ChronoDateTimeUtc,

	// TODO: origin, result, instrument
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl activitystream::Object for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn full_type(&self) -> Option<activitystream::Type> {
		Some(activitystream::Type::ActivityType(self.activity_type))
	}

	fn published(&self) -> Option<chrono::DateTime<chrono::Utc>> {
		Some(self.published)
	}
}

impl activitystream::Activity for Model {
	fn activity_type(&self) -> Option<activitystream::types::ActivityType> {
		Some(self.activity_type)
	}

	fn actor_id(&self) -> Option<&str> {
		Some(&self.actor)
	}

	fn object_id(&self) -> Option<&str> {
		self.object.as_deref()
	}

	fn target(&self) -> Option<&str> {
		self.target.as_deref()
	}
}
