use apb::{BaseMut, Collection, CollectionMut, ObjectMut};
use sea_orm::entity::prelude::*;

use crate::routes::activitypub::jsonld::LD;

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
	pub in_reply_to: Option<String>,
	pub cc: Audience,
	pub bcc: Audience,
	pub to: Audience,
	pub bto: Audience,
	pub url: Option<String>,
	pub published: ChronoDateTimeUtc,
	pub updated: Option<ChronoDateTimeUtc>,

	pub sensitive: bool,
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
			in_reply_to: object.in_reply_to().id(),
			published: object.published().ok_or(super::FieldError("published"))?,
			updated: object.updated(),
			url: object.url().id(),
			comments: object.replies().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i64,
			likes: object.likes().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i64,
			shares: object.shares().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i64,
			to: object.to().into(),
			bto: object.bto().into(),
			cc: object.cc().into(),
			bcc: object.bcc().into(),

			sensitive: object.sensitive().unwrap_or(false),
		})
	}

	pub fn ap(self) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(&self.id))
			.set_object_type(Some(self.object_type))
			.set_attributed_to(apb::Node::maybe_link(self.attributed_to))
			.set_name(self.name.as_deref())
			.set_summary(self.summary.as_deref())
			.set_content(self.content.as_deref())
			.set_context(apb::Node::maybe_link(self.context.clone()))
			.set_conversation(apb::Node::maybe_link(self.context.clone())) // duplicate context for mastodon
			.set_in_reply_to(apb::Node::maybe_link(self.in_reply_to.clone()))
			.set_published(Some(self.published))
			.set_updated(self.updated)
			.set_to(apb::Node::links(self.to.0.clone()))
			.set_bto(apb::Node::Empty)
			.set_cc(apb::Node::links(self.cc.0.clone()))
			.set_bcc(apb::Node::Empty)
			.set_url(apb::Node::maybe_link(self.url))
			.set_sensitive(Some(self.sensitive))
			.set_shares(apb::Node::object(
				serde_json::Value::new_object()
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.shares as u64))
			))
			.set_likes(apb::Node::object(
				serde_json::Value::new_object()
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.likes as u64))
			))
			.set_replies(apb::Node::object(
				serde_json::Value::new_object()
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.comments as u64))
			))
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

	#[sea_orm(has_many = "super::attachment::Entity")]
	Attachment,

	#[sea_orm(has_many = "super::like::Entity")]
	Like,

	#[sea_orm(has_many = "super::share::Entity")]
	Share,
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

impl Related<super::attachment::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Attachment.def()
	}
}

impl Related<super::like::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Like.def()
	}
}

impl Related<super::share::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Share.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl apb::target::Addressed for Model {
	fn addressed(&self) -> Vec<String> {
		let mut to : Vec<String> = self.to.0.clone();
		to.append(&mut self.bto.0.clone());
		to.append(&mut self.cc.0.clone());
		to.append(&mut self.bcc.0.clone());
		to
	}
}
