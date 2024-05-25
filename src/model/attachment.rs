use apb::{DocumentMut, ObjectMut};
use sea_orm::entity::prelude::*;

use crate::routes::activitypub::jsonld::LD;

use super::addressing::Event;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "attachments")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub url: String,
	pub object: i64,
	pub document_type: String,
	pub name: Option<String>,
	pub media_type: String,
	pub created: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Internal",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Objects,
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

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
