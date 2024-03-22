use sea_orm::entity::prelude::*;

use crate::{activitypub::jsonld::LD, activitystream::{object::{ObjectMut, ObjectType}, BaseMut, BaseType, Link, Node}};

use super::Audience;

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
	pub likes: u64,
	pub shares: u64,
	pub comments: u64,
	pub context: Option<String>,
	pub cc: Audience,
	pub bcc: Audience,
	pub to: Audience,
	pub bto: Audience,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::activity::Entity")]
	Activity,

	#[sea_orm(
		belongs_to = "super::user::Entity",
		from = "Column::AttributedTo",
		to = "super::user::Column::Id",
	)]
	User,
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Activity.def()
	}
}

impl Related<super::user::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl crate::activitystream::Base for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Object(self.object_type))
	}

	fn underlying_json_object(self) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(&self.id))
			.set_object_type(Some(self.object_type))
			.set_attributed_to(Node::maybe_link(self.attributed_to))
			.set_name(self.name.as_deref())
			.set_summary(self.summary.as_deref())
			.set_content(self.content.as_deref())
			.set_context(Node::maybe_link(self.context.clone()))
			.set_published(Some(self.published))
			.set_to(Node::links(self.to.0.clone()))
			.set_bto(Node::empty())
			.set_cc(Node::links(self.cc.0.clone()))
			.set_bcc(Node::empty())
	}
}

impl crate::activitystream::object::Object for Model {
	fn object_type(&self) -> Option<ObjectType> {
		Some(self.object_type)
	}

	fn attributed_to(&self) -> Node<impl crate::activitystream::object::actor::Actor> {
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

	fn context(&self) -> Node<impl crate::activitystream::Object> {
		Node::maybe_link(self.context.clone())
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

	fn published (&self) -> Option<chrono::DateTime<chrono::Utc>> {
		Some(self.published)
	}
}

impl Model {
	pub fn new(object: &impl crate::activitystream::object::Object) -> Result<Self, super::FieldError> {
		Ok(Model {
			id: object.id().ok_or(super::FieldError("id"))?.to_string(),
			object_type: object.object_type().ok_or(super::FieldError("type"))?,
			attributed_to: object.attributed_to().id().map(|x| x.to_string()),
			name: object.name().map(|x| x.to_string()),
			summary: object.summary().map(|x| x.to_string()),
			content: object.content().map(|x| x.to_string()),
			context: object.context().id().map(|x| x.to_string()),
			published: object.published().ok_or(super::FieldError("published"))?,
			comments: 0,
			likes: 0,
			shares: 0,
			to: object.to().into(),
			bto: object.bto().into(),
			cc: object.cc().into(),
			bcc: object.bcc().into(),
		})
	}
}
