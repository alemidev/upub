use apb::{ActivityMut, BaseMut, ObjectMut};
use sea_orm::entity::prelude::*;

use crate::routes::activitypub::jsonld::LD;

use super::Audience;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "activities")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: String,

	pub activity_type: apb::ActivityType,
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
	pub fn new(activity: &impl apb::Activity) -> Result<Self, super::FieldError> {
		Ok(Model {
			id: activity.id().ok_or(super::FieldError("id"))?.to_string(),
			activity_type: activity.activity_type().ok_or(super::FieldError("type"))?,
			actor: activity.actor().id().ok_or(super::FieldError("actor"))?,
			object: activity.object().id(),
			target: activity.target().id(),
			published: activity.published().unwrap_or(chrono::Utc::now()),
			to: activity.to().into(),
			bto: activity.bto().into(),
			cc: activity.cc().into(),
			bcc: activity.bcc().into(),
		})
	}

	pub fn ap(self) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(&self.id))
			.set_activity_type(Some(self.activity_type))
			.set_actor(apb::Node::link(self.actor))
			.set_object(apb::Node::maybe_link(self.object))
			.set_target(apb::Node::maybe_link(self.target))
			.set_published(Some(self.published))
			.set_to(apb::Node::links(self.to.0.clone()))
			.set_bto(apb::Node::Empty)
			.set_cc(apb::Node::links(self.cc.0.clone()))
			.set_bcc(apb::Node::Empty)
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

	#[sea_orm(has_many = "super::delivery::Entity")]
	Delivery,
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
