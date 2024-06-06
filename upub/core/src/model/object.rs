use apb::{field::OptionalString, BaseMut, Collection, CollectionMut, ObjectMut, ObjectType};
use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

use super::Audience;

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
	pub sensitive: bool,
	pub in_reply_to: Option<String>,
	pub url: Option<String>,
	pub likes: i32,
	pub announces: i32,
	pub replies: i32,
	pub context: Option<String>,
	pub to: Audience,
	pub bto: Audience,
	pub cc: Audience,
	pub bcc: Audience,
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
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Objects,
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
		Relation::Objects.def()
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

impl ActiveModel {
	#[deprecated = "use AP::object() from processor::normalize"]
	pub fn new(object: &impl apb::Object) -> Result<Self, apb::FieldErr> {
		let t = object.object_type()?;
		if matches!(t,
			apb::ObjectType::Activity(_)
			| apb::ObjectType::Actor(_)
			| apb::ObjectType::Collection(_)
			| apb::ObjectType::Document(_)
		) {
			return Err(apb::FieldErr("type"));
		}
		Ok(ActiveModel {
			internal: sea_orm::ActiveValue::NotSet,
			id: sea_orm::ActiveValue::Set(object.id()?.to_string()),
			object_type: sea_orm::ActiveValue::Set(t),
			attributed_to: sea_orm::ActiveValue::Set(object.attributed_to().id().str()),
			name: sea_orm::ActiveValue::Set(object.name().str()),
			summary: sea_orm::ActiveValue::Set(object.summary().str()),
			content: sea_orm::ActiveValue::Set(object.content().str()),
			context: sea_orm::ActiveValue::Set(object.context().id().str()),
			in_reply_to: sea_orm::ActiveValue::Set(object.in_reply_to().id().str()),
			published: sea_orm::ActiveValue::Set(object.published().unwrap_or_else(|_| chrono::Utc::now())),
			updated: sea_orm::ActiveValue::Set(object.updated().unwrap_or_else(|_| chrono::Utc::now())),
			url: sea_orm::ActiveValue::Set(object.url().id().str()),
			replies: sea_orm::ActiveValue::Set(object.replies().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i32),
			likes: sea_orm::ActiveValue::Set(object.likes().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i32),
			announces: sea_orm::ActiveValue::Set(object.shares().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i32),
			to: sea_orm::ActiveValue::Set(object.to().into()),
			bto: sea_orm::ActiveValue::Set(object.bto().into()),
			cc: sea_orm::ActiveValue::Set(object.cc().into()),
			bcc: sea_orm::ActiveValue::Set(object.bcc().into()),

			sensitive: sea_orm::ActiveValue::Set(object.sensitive().unwrap_or(false)),
		})
	}
}

impl Model {
	pub fn ap(self) -> serde_json::Value {
		apb::new()
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
			.set_updated(Some(self.updated))
			.set_audience(apb::Node::maybe_link(self.audience))
			.set_to(apb::Node::links(self.to.0.clone()))
			.set_bto(apb::Node::Empty)
			.set_cc(apb::Node::links(self.cc.0.clone()))
			.set_bcc(apb::Node::Empty)
			.set_url(apb::Node::maybe_link(self.url))
			.set_sensitive(Some(self.sensitive))
			.set_shares(apb::Node::object(
				apb::new()
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.announces as u64))
			))
			.set_likes(apb::Node::object(
				apb::new()
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.likes as u64))
			))
			.set_replies(apb::Node::object(
				apb::new()
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.replies as u64))
			))
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
}
