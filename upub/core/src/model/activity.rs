use apb::{ActivityMut, ActivityType, BaseMut, ObjectMut};
use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

use crate::ext::JsonVec;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "activities")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub id: String,
	pub activity_type: ActivityType,
	pub actor: String,
	pub object: Option<String>,
	pub target: Option<String>,
	pub to: JsonVec<String>,
	pub bto: JsonVec<String>,
	pub cc: JsonVec<String>,
	pub bcc: JsonVec<String>,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Actor",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Actors,
	#[sea_orm(has_many = "super::addressing::Entity")]
	Addressing,
	#[sea_orm(has_many = "super::notification::Entity")]
	Notifications,
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Id",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Objects,
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

impl Related<super::notification::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Notifications.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
	pub fn find_by_ap_id(id: &str) -> Select<Entity> {
		Entity::find().filter(Column::Id.eq(id))
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
			.set_id(Some(self.id))
			.set_activity_type(Some(self.activity_type))
			.set_actor(apb::Node::link(self.actor))
			.set_object(apb::Node::maybe_link(self.object))
			.set_target(apb::Node::maybe_link(self.target))
			.set_published(Some(self.published))
			.set_to(apb::Node::links(self.to.0.clone()))
			.set_bto(apb::Node::Empty)
			.set_cc(apb::Node::links(self.cc.0.clone()))
			.set_bcc(apb::Node::Empty)
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
