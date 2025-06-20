use apb::{BaseMut, CollectionMut, ObjectMut};
use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lists")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub id: String,
	pub attributed_to: String,
	pub name: Option<String>,
	pub summary: Option<String>,
	pub published: ChronoDateTimeUtc,
	pub updated: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::AttributedTo",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Actors,
	#[sea_orm(has_many = "super::list_element::Entity")]
	ListElements,
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl Related<super::list_element::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ListElements.def()
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
	fn into_activity_pub_json(self, _ctx: &crate::Context) -> serde_json::Value {
		apb::new()
			.set_id(Some(self.id.clone()))
			.set_collection_type(Some(apb::CollectionType::Collection))
			.set_name(self.name)
			.set_summary(self.summary)
			.set_attributed_to(apb::Node::link(self.attributed_to))
			.set_published(Some(self.published))
			.set_updated(if self.updated != self.published { Some(self.updated) } else { None })
			.set_first(apb::Node::link(format!("{}/page", self.id))) // TODO not really generic
	}
}
