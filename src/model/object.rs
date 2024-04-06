use sea_orm::entity::prelude::*;

use super::Audience;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "objects")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: String,
	pub object_type: apb::ObjectType,
	pub attributed_to: Option<String>,
	pub name: Option<String>,
	pub summary: Option<String>,
	pub content: Option<String>,
	pub likes: i64,
	pub shares: i64,
	pub comments: i64,
	pub context: Option<String>,
	pub cc: Audience,
	pub bcc: Audience,
	pub to: Audience,
	pub bto: Audience,
	pub published: ChronoDateTimeUtc,
}

impl Model {
	pub fn new(object: &impl apb::Object) -> Result<Self, super::FieldError> {
		Ok(Model {
			id: object.id().ok_or(super::FieldError("id"))?.to_string(),
			object_type: object.object_type().ok_or(super::FieldError("type"))?,
			attributed_to: object.attributed_to().id(),
			name: object.name().map(|x| x.to_string()),
			summary: object.summary().map(|x| x.to_string()),
			content: object.content().map(|x| x.to_string()),
			context: object.context().id(),
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

	#[sea_orm(has_many = "super::addressing::Entity")]
	Addressing,
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

impl Related<super::addressing::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Addressing.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
