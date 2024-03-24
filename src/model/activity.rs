use sea_orm::entity::prelude::*;

use crate::activitystream::object::activity::{Activity, ActivityType};

use super::Audience;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "activities")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: String,

	pub activity_type: ActivityType,
	pub actor: String,
	pub object: Option<String>,

	pub target: Option<String>, // TODO relates to USER maybe??
	pub cc: Audience,
	pub bcc: Audience,
	pub to: Audience,
	pub bto: Audience,
	pub published: ChronoDateTimeUtc,

	// TODO: origin, result, instrument
}

impl Model {
	pub fn new(activity: &impl Activity) -> Result<Self, super::FieldError> {
		Ok(Model {
			id: activity.id().ok_or(super::FieldError("id"))?.to_string(),
			activity_type: activity.activity_type().ok_or(super::FieldError("type"))?,
			actor: activity.actor().id().ok_or(super::FieldError("actor"))?.to_string(),
			object: activity.object().id().map(|x| x.to_string()),
			target: activity.target().id().map(|x| x.to_string()),
			published: activity.published().unwrap_or(chrono::Utc::now()),
			to: activity.to().into(),
			bto: activity.bto().into(),
			cc: activity.cc().into(),
			bcc: activity.bcc().into(),
		})
	}
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::user::Entity",
		from = "Column::Actor",
		to = "super::user::Column::Id"
	)]
	User,

	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Id"
	)]
	Object,

	#[sea_orm(has_many = "super::addressing::Entity")]
	Addressing,
}

impl Related<super::user::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Object.def()
	}
}

impl Related<super::addressing::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Addressing.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
