use apb::{ActivityMut, ObjectMut};
use sea_orm::{entity::prelude::*, Condition, FromQueryResult, Iterable, Order, QueryOrder, QuerySelect, SelectColumns};

use crate::routes::activitypub::jsonld::LD;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "addressing")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i64,
	pub actor: String,
	pub server: String,
	pub activity: Option<String>,
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



#[allow(clippy::large_enum_variant)] // tombstone is an outlier, not the norm! this is a beefy enum
#[derive(Debug, Clone)]
pub enum Event {
	Tombstone,
	StrayObject(crate::model::object::Model),
	Activity(crate::model::activity::Model),
	DeepActivity {
		activity: crate::model::activity::Model,
		object: crate::model::object::Model,
	}
}


impl Event {
	pub fn id(&self) -> &str {
		match self {
			Event::Tombstone => "",
			Event::StrayObject(x) => x.id.as_str(),
			Event::Activity(x) => x.id.as_str(),
			Event::DeepActivity { activity: _, object } => object.id.as_str(),
		}
	}

	pub fn ap(self, attachment: Option<Vec<crate::model::attachment::Model>>) -> serde_json::Value {
		let attachment = match attachment {
			None => apb::Node::Empty,
			Some(vec) => apb::Node::array(
				vec.into_iter().map(|x| x.ap()).collect()
			),
		};
		match self {
			Event::Activity(x) => x.ap(),
			Event::DeepActivity { activity, object } =>
				activity.ap().set_object(apb::Node::object(object.ap().set_attachment(attachment))),
			Event::StrayObject(x) => serde_json::Value::new_object()
				.set_activity_type(Some(apb::ActivityType::Activity))
				.set_object(apb::Node::object(x.ap().set_attachment(attachment))),
			Event::Tombstone => serde_json::Value::new_object()
				.set_activity_type(Some(apb::ActivityType::Activity))
				.set_object(apb::Node::object(
					serde_json::Value::new_object()
						.set_object_type(Some(apb::ObjectType::Tombstone))
				)),
		}
	}
}

impl FromQueryResult for Event {
	fn from_query_result(res: &sea_orm::QueryResult, _pre: &str) -> Result<Self, sea_orm::DbErr> {
		let activity = crate::model::activity::Model::from_query_result(res, crate::model::activity::Entity.table_name()).ok();
		let object = crate::model::object::Model::from_query_result(res, crate::model::object::Entity.table_name()).ok();
		match (activity, object) {
			(Some(activity), Some(object)) => Ok(Self::DeepActivity { activity, object }),
			(Some(activity), None) => Ok(Self::Activity(activity)),
			(None, Some(object)) => Ok(Self::StrayObject(object)),
			(None, None) => Ok(Self::Tombstone),
		}
	}
}


impl Entity {
	pub fn find_addressed() -> Select<Entity> {
		let mut select = Entity::find()
			.distinct()
			.select_only()
			.join(sea_orm::JoinType::LeftJoin, Relation::Object.def())
			.join(sea_orm::JoinType::LeftJoin, Relation::Activity.def())
			.filter(
				// TODO ghetto double inner join because i want to filter out tombstones
				Condition::any()
					.add(crate::model::activity::Column::Id.is_not_null())
					.add(crate::model::object::Column::Id.is_not_null())
			)
			.order_by(Column::Published, Order::Desc);

		for col in crate::model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::object::Entity.table_name(), col.to_string()));
		}

		for col in crate::model::activity::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::activity::Entity.table_name(), col.to_string()));
		}

		select
	}
}
