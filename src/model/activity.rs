use sea_orm::entity::prelude::*;

use crate::activitystream::{node::{InsertStr, Node}, object::{activity::{Activity, ActivityType}, actor::Actor, Object, ObjectType}, Base, BaseType};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "activities")]
pub struct Model {
	#[sea_orm(primary_key)]
	/// must be https://instance.org/users/:user , even if local! TODO bad design...
	pub id: String,

	pub activity_type: ActivityType,
	pub actor: String, // TODO relates to USER
	pub object: Option<String>, // TODO relates to NOTES maybe????? maybe other tables??????
	pub target: Option<String>, // TODO relates to USER maybe??
	pub published: ChronoDateTimeUtc,

	// TODO: origin, result, instrument
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Base for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Object(ObjectType::Activity(self.activity_type)))
	}
}

impl Object for Model {
	fn published(&self) -> Option<chrono::DateTime<chrono::Utc>> {
		Some(self.published)
	}
}

impl Activity for Model {
	fn activity_type(&self) -> Option<ActivityType> {
		Some(self.activity_type)
	}

	fn actor(&self) -> Node<impl Actor> {
		Node::<serde_json::Value>::Link(Box::new(self.actor.clone()))
	}

	fn object(&self) -> Node<impl Object> {
		match &self.object {
			None => Node::Empty::<serde_json::Value>,
			Some(x) => Node::Link(Box::new(x.clone())),
		}
	}

	fn target(&self) -> Option<&str> {
		self.target.as_deref()
	}
}

impl Model {
	pub fn new(activity: &impl Activity) -> Result<Self, super::FieldError> {
		Ok(Model {
			id: activity.id().ok_or(super::FieldError("id"))?.to_string(),
			activity_type: activity.activity_type().ok_or(super::FieldError("type"))?,
			actor: activity.actor().id().ok_or(super::FieldError("actor"))?.to_string(),
			object: activity.object().id().map(|x| x.to_string()),
			target: activity.target().map(|x| x.to_string()),
			published: activity.published().ok_or(super::FieldError("published"))?,
		})
	}
}

impl super::ToJson for Model {
	fn json(&self) -> serde_json::Value {
		let mut map = serde_json::Map::new();
		map.insert_str("id", Some(&self.id));
		map.insert_str("type", Some(self.activity_type.as_ref()));
		map.insert_str("actor", Some(&self.actor));
		map.insert_str("object", self.object.as_deref());
		map.insert_str("target", self.target.as_deref());
		map.insert_timestr("published", Some(self.published));
		serde_json::Value::Object(map)
	}
}
