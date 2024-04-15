use apb::{ActivityMut, Node};
use sea_orm::{entity::prelude::*, FromQueryResult, Iterable, QuerySelect, SelectColumns};

use crate::routes::activitypub::{activity::ap_activity, object::ap_object};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "addressing")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i64,
	pub actor: String,
	pub server: String,
	pub activity: String,
	pub object: Option<String>,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::user::Entity",
		from = "Column::Actor",
		to = "super::user::Column::Id"
	)]
	User,

	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Activity",
		to = "super::activity::Column::Id"
	)]
	Activity,

	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Id"
	)]
	Object,
}

impl Related<super::user::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Activity.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Object.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct EmbeddedActivity {
	pub activity: crate::model::activity::Model,
	pub object: Option<crate::model::object::Model>,
}

impl From<EmbeddedActivity> for serde_json::Value {
	fn from(value: EmbeddedActivity) -> Self {
		match value.object {
			Some(o) => ap_activity(value.activity).set_object(Node::object(ap_object(o))),
			None => ap_activity(value.activity)
		}
	}
}

impl FromQueryResult for EmbeddedActivity {
	fn from_query_result(res: &sea_orm::QueryResult, _pre: &str) -> Result<Self, sea_orm::DbErr> {
		let activity = crate::model::activity::Model::from_query_result(res, crate::model::activity::Entity.table_name())?;
		let object = crate::model::object::Model::from_query_result(res, crate::model::object::Entity.table_name()).ok();
		Ok(Self { activity, object })
	}
}

impl Entity {
	pub fn find_activities() -> Select<Entity> {
		let mut select = Entity::find()
			.distinct()
			.select_only()
			.join(sea_orm::JoinType::InnerJoin, Relation::Activity.def())
			// INNERJOIN: filter out addressings for which we don't have an activity anymore
			// TODO we could in theory return just the link or fetch them again, just ignoring them is mehh
			.join(sea_orm::JoinType::LeftJoin, crate::model::activity::Relation::Object.def());

		for col in crate::model::activity::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::activity::Entity.table_name(), col.to_string()));
		}

		for col in crate::model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::object::Entity.table_name(), col.to_string()));
		}

		select
	}

	pub fn find_objects() -> Select<Entity> {
		let mut select = Entity::find()
			.distinct()
			.select_only()
			.join(sea_orm::JoinType::InnerJoin, Relation::Object.def());
			// INNERJOIN: filter out addressings for which we don't have an activity anymore
			// TODO we could in theory return just the link or fetch them again, just ignoring them is mehh

		for col in crate::model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::object::Entity.table_name(), col.to_string()));
		}

		select
	}
}
