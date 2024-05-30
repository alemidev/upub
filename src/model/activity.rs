use apb::{ActivityMut, ActivityType, BaseMut, ObjectMut};
use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

use crate::{model::Audience, errors::UpubError, routes::activitypub::jsonld::LD};

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
	pub to: Audience,
	pub bto: Audience,
	pub cc: Audience,
	pub bcc: Audience,
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
	#[sea_orm(has_many = "super::delivery::Entity")]
	Deliveries,
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

impl Related<super::delivery::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Deliveries.def()
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

	pub async fn ap_to_internal(id: &str, db: &DatabaseConnection) -> crate::Result<i64> {
		Entity::find()
			.filter(Column::Id.eq(id))
			.select_only()
			.select_column(Column::Internal)
			.into_tuple::<i64>()
			.one(db)
			.await?
			.ok_or_else(UpubError::not_found)
	}
}

impl ActiveModel {
	//#[deprecated = "should remove this, get models thru normalizer"]
	pub fn new(activity: &impl apb::Activity) -> Result<Self, super::FieldError> {
		Ok(ActiveModel {
			internal: sea_orm::ActiveValue::NotSet,
			id: sea_orm::ActiveValue::Set(activity.id().ok_or(super::FieldError("id"))?.to_string()),
			activity_type: sea_orm::ActiveValue::Set(activity.activity_type().ok_or(super::FieldError("type"))?),
			actor: sea_orm::ActiveValue::Set(activity.actor().id().ok_or(super::FieldError("actor"))?),
			object: sea_orm::ActiveValue::Set(activity.object().id()),
			target: sea_orm::ActiveValue::Set(activity.target().id()),
			published: sea_orm::ActiveValue::Set(activity.published().unwrap_or(chrono::Utc::now())),
			to: sea_orm::ActiveValue::Set(activity.to().into()),
			bto: sea_orm::ActiveValue::Set(activity.bto().into()),
			cc: sea_orm::ActiveValue::Set(activity.cc().into()),
			bcc: sea_orm::ActiveValue::Set(activity.bcc().into()),
		})
	}
}

impl Model {
	pub fn ap(self) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(&self.id))
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
}
