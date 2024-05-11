use apb::{Document, DocumentMut, Link, Object, ObjectMut};
use sea_orm::{entity::prelude::*, Set};

use crate::routes::activitypub::jsonld::LD;

use super::addressing::Event;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "attachments")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i64,

	pub url: String,
	pub object: String,
	pub document_type: apb::DocumentType,
	pub name: Option<String>,
	pub media_type: String,
	pub created: ChronoDateTimeUtc,
}

impl ActiveModel {
	// TODO receive an impl, not a specific type!
	// issue is that it's either an apb::Link or apb::Document, but Document doesnt inherit from link!
	pub fn new(document: &serde_json::Value, object: String, media_type: Option<String>) -> Result<ActiveModel, super::FieldError> {
		let media_type = media_type.unwrap_or_else(|| document.media_type().unwrap_or("link").to_string());
		Ok(ActiveModel {
			id: sea_orm::ActiveValue::NotSet,
			object: Set(object),
			url: Set(document.url().id().unwrap_or_else(|| document.href().to_string())),
			document_type: Set(document.document_type().unwrap_or(apb::DocumentType::Page)),
			media_type: Set(media_type),
			name: Set(document.name().map(|x| x.to_string())),
			created: Set(document.published().unwrap_or(chrono::Utc::now())),
		})
	}
}

impl Model {
	pub fn ap(self) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_url(apb::Node::link(self.url))
			.set_document_type(Some(self.document_type))
			.set_media_type(Some(&self.media_type))
			.set_name(self.name.as_deref())
			.set_published(Some(self.created))
	}
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Id"
	)]
	Object,
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Object.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}


#[axum::async_trait]
pub trait BatchFillable {
	async fn load_attachments_batch(&self, db: &DatabaseConnection) -> Result<std::collections::BTreeMap<String, Vec<Model>>, DbErr>;
}

#[axum::async_trait]
impl BatchFillable for &[Event] {
	async fn load_attachments_batch(&self, db: &DatabaseConnection) -> Result<std::collections::BTreeMap<String, Vec<Model>>, DbErr> {
		let objects : Vec<crate::model::object::Model> = self
			.iter()
			.filter_map(|x| match x {
				Event::Tombstone => None,
				Event::Activity(_) => None,
				Event::StrayObject { object, liked: _ } => Some(object.clone()),
				Event::DeepActivity { activity: _, liked: _, object } => Some(object.clone()),
			})
			.collect();

		let attachments = objects.load_many(Entity, db).await?;

		let mut out : std::collections::BTreeMap<String, Vec<Model>> = std::collections::BTreeMap::new();
		for attach in attachments.into_iter().flatten() {
			if out.contains_key(&attach.object) {
				out.get_mut(&attach.object).expect("contains but get failed?").push(attach);
			} else {
				out.insert(attach.object.clone(), vec![attach]);
			}
		}

		Ok(out)
	}
}

#[axum::async_trait]
impl BatchFillable for Vec<Event> {
	async fn load_attachments_batch(&self, db: &DatabaseConnection) -> Result<std::collections::BTreeMap<String, Vec<Model>>, DbErr> {
		self.as_slice().load_attachments_batch(db).await
	}
}

#[axum::async_trait]
impl BatchFillable for Event {
	async fn load_attachments_batch(&self, db: &DatabaseConnection) -> Result<std::collections::BTreeMap<String, Vec<Model>>, DbErr> {
		let x = vec![self.clone()]; // TODO wasteful clone and vec![] but ehhh convenient
		x.load_attachments_batch(db).await
	}
}
