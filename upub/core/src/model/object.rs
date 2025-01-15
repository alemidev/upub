use apb::{BaseMut, CollectionMut, DocumentMut, ObjectMut, ObjectType};
use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

use crate::ext::JsonVec;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "objects")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub id: String,
	pub object_type: ObjectType,
	pub attributed_to: Option<String>,
	pub name: Option<String>,
	pub summary: Option<String>,
	pub content: Option<String>,
	pub image: Option<String>,
	pub quote: Option<String>,
	pub sensitive: bool,
	pub in_reply_to: Option<String>,
	pub url: Option<String>,
	pub likes: i32,
	pub announces: i32,
	pub replies: i32,
	pub context: Option<String>,
	pub to: JsonVec<String>,
	pub bto: JsonVec<String>,
	pub cc: JsonVec<String>,
	pub bcc: JsonVec<String>,
	pub published: ChronoDateTimeUtc,
	pub updated: ChronoDateTimeUtc,

	pub audience: Option<String>, // added with migration m20240606_000001
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::activity::Entity")]
	Activities,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::AttributedTo",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Actors,
	#[sea_orm(has_many = "super::addressing::Entity")]
	Addressing,
	#[sea_orm(has_many = "super::announce::Entity")]
	Announces,
	#[sea_orm(has_many = "super::attachment::Entity")]
	Attachments,
	#[sea_orm(has_many = "super::dislike::Entity")]
	Dislikes,
	#[sea_orm(has_many = "super::hashtag::Entity")]
	Hashtags,
	#[sea_orm(has_many = "super::like::Entity")]
	Likes,
	#[sea_orm(has_many = "super::mention::Entity")]
	Mentions,
	#[sea_orm(
		belongs_to = "Entity",
		from = "Column::InReplyTo",
		to = "Column::Id",
		on_update = "NoAction",
		on_delete = "NoAction"
	)]
	ObjectsReply,
	#[sea_orm(
		belongs_to = "Entity",
		from = "Column::Quote",
		to = "Column::Id",
		on_update = "NoAction",
		on_delete = "NoAction"
	)]
	ObjectsQuote,
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Activities.def()
	}
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl Related<super::addressing::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Addressing.def()
	}
}

impl Related<super::announce::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Announces.def()
	}
}

impl Related<super::attachment::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Attachments.def()
	}
}

impl Related<super::dislike::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Dislikes.def()
	}
}

impl Related<super::hashtag::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Hashtags.def()
	}
}

impl Related<super::like::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Likes.def()
	}
}

impl Related<super::mention::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Mentions.def()
	}
}

impl Related<Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ObjectsReply.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
	pub fn find_by_ap_id(id: &str) -> Select<Entity> {
		Entity::find().filter(Column::Id.eq(id))
	}

	pub fn delete_by_ap_id(id: &str) -> sea_orm::DeleteMany<Entity> {
		Entity::delete_many().filter(Column::Id.eq(id))
	}

	pub async fn ap_to_internal(id: &str, db: &impl ConnectionTrait) -> Result<Option<i64>, DbErr> {
		Entity::find()
			.filter(Column::Id.eq(id))
			.select_only()
			.select_column(Column::Internal)
			.into_tuple::<i64>()
			.one(db)
			.await
	}
}

impl crate::ext::IntoActivityPub for Model {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value {
		let is_local = ctx.is_local(&self.id);
		apb::new()
			.set_object_type(Some(self.object_type))
			.set_attributed_to(apb::Node::maybe_link(self.attributed_to))
			.set_name(self.name)
			.set_summary(self.summary)
			.set_content(self.content)
			.set_image(apb::Node::maybe_object(self.image.map(|x| 
				apb::new()
					.set_document_type(Some(apb::DocumentType::Image))
					.set_url(apb::Node::link(x))
			)))
			.set_context(apb::Node::maybe_link(self.context.clone()))
			.set_conversation(apb::Node::maybe_link(self.context)) // duplicate context for mastodon
			.set_in_reply_to(apb::Node::maybe_link(self.in_reply_to.clone()))
			.set_quote_url(apb::Node::maybe_link(self.quote.clone()))
			.set_published(Some(self.published))
			.set_updated(if self.updated != self.published { Some(self.updated) } else { None })
			.set_audience(apb::Node::maybe_link(self.audience))
			.set_to(apb::Node::links(self.to.0.clone()))
			.set_bto(apb::Node::Empty)
			.set_cc(apb::Node::links(self.cc.0.clone()))
			.set_bcc(apb::Node::Empty)
			.set_url(apb::Node::maybe_link(self.url))
			.set_sensitive(Some(self.sensitive))
			.set_shares(apb::Node::object(
				apb::new()
					.set_id(if is_local { Some(format!("{}/shares", self.id)) } else { None })
					.set_first(if is_local { apb::Node::link(format!("{}/shares/page", self.id)) } else { apb::Node::Empty })
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.announces as u64))
			))
			.set_likes(apb::Node::object(
				apb::new()
					.set_id(if is_local { Some(format!("{}/likes", self.id)) } else { None })
					.set_first(if is_local { apb::Node::link(format!("{}/likes/page", self.id)) } else { apb::Node::Empty })
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.likes as u64))
			))
			.set_replies(apb::Node::object(
				apb::new()
					.set_id(if is_local { Some(format!("{}/replies", self.id)) } else { None })
					.set_first( if is_local { apb::Node::link(format!("{}/replies/page", self.id)) } else { apb::Node::Empty })
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.replies as u64))
			))
			.set_id(Some(self.id))
	}
}

impl apb::target::Addressed for Model {
	fn addressed(&self) -> Vec<String> {
		let mut to : Vec<String> = self.to.0.clone();
		to.append(&mut self.bto.0.clone());
		to.append(&mut self.cc.0.clone());
		to.append(&mut self.bcc.0.clone());
		to
	}

	fn mentioning(&self) -> Vec<String> {
		let mut to = self.to.0.clone();
		to.append(&mut self.bto.0.clone());
		to
	}
}
