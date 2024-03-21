use sea_orm::{entity::prelude::*, FromJsonQueryResult};

use crate::activitystream::{self, link::Link, object::{activity::{Activity, ActivityMut, ActivityType}, actor::Actor, Object, ObjectMut, ObjectType}, Base, BaseMut, BaseType, Node};

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, FromJsonQueryResult)]
pub struct Audience(pub Vec<String>);

impl<T : Link> From<Node<T>> for Audience {
	fn from(value: Node<T>) -> Self {
		Audience(
			match value {
				Node::Empty => vec![],
				Node::Link(l) => vec![l.href().to_string()],
				Node::Object(o) => if let Some(id) = o.id() { vec![id.to_string()] } else { vec![] },
				Node::Array(arr) => arr.into_iter().filter_map(|l| Some(l.id()?.to_string())).collect(),
			}
		)
	}
}

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
	pub cc: Audience,
	pub bcc: Audience,
	pub to: Audience,
	pub bto: Audience,
	pub published: ChronoDateTimeUtc,

	// TODO: origin, result, instrument
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

impl ActiveModelBehavior for ActiveModel {}

impl Base for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Object(ObjectType::Activity(self.activity_type)))
	}

	fn underlying_json_object(self) -> serde_json::Value {
		activitystream::object()
			.set_id(Some(&self.id))
			.set_activity_type(Some(self.activity_type))
			.set_actor(Node::link(self.actor))
			.set_object(Node::maybe_link(self.object))
			.set_target(Node::maybe_link(self.target))
			.set_published(Some(self.published))
			.set_to(Node::links(self.to.0.clone()))
			.set_bto(Node::empty())
			.set_cc(Node::links(self.cc.0.clone()))
			.set_bcc(Node::empty())
	}
}

impl Object for Model {
	fn object_type(&self) -> Option<ObjectType> {
		Some(ObjectType::Activity(self.activity_type))
	}

	fn published(&self) -> Option<chrono::DateTime<chrono::Utc>> {
		Some(self.published)
	}

	fn to(&self) -> Node<impl Link> {
		Node::links(self.to.0.clone())
	}

	fn bto(&self) -> Node<impl Link> {
		Node::links(self.bto.0.clone())
	}

	fn cc(&self) -> Node<impl Link> {
		Node::links(self.cc.0.clone())
	}

	fn bcc(&self) -> Node<impl Link> {
		Node::links(self.bcc.0.clone())
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

	fn target(&self) -> Node<impl Object> {
		match &self.target {
			None => Node::Empty::<serde_json::Value>,
			Some(x) => Node::Link(Box::new(x.clone())),
		}
	}
}

impl Model {
	pub fn new(activity: &impl Activity) -> Result<Self, super::FieldError> {
		Ok(Model {
			id: activity.id().ok_or(super::FieldError("id"))?.to_string(),
			activity_type: activity.activity_type().ok_or(super::FieldError("type"))?,
			actor: activity.actor().id().ok_or(super::FieldError("actor"))?.to_string(),
			object: activity.object().id().map(|x| x.to_string()),
			target: activity.target().id().map(|x| x.to_string()),
			published: activity.published().ok_or(super::FieldError("published"))?,
			to: activity.to().into(),
			bto: activity.bto().into(),
			cc: activity.cc().into(),
			bcc: activity.bcc().into(),
		})
	}
}
