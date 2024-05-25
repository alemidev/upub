use apb::{ActivityMut, ObjectMut};
use sea_orm::{entity::prelude::*, sea_query::IntoCondition, Condition, FromQueryResult, Iterable, Order, QueryOrder, QuerySelect, SelectColumns};

use crate::routes::activitypub::jsonld::LD;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "addressing")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub actor: Option<i64>,
	pub instance: Option<i64>,
	pub activity: Option<i64>,
	pub object: Option<i64>,
	pub published: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Activity",
		to = "super::activity::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Activities,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Actor",
		to = "super::actor::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Actors,
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::Instance",
		to = "super::instance::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Instances,
	#[sea_orm(
		belongs_to = "super::object::Entity",
		from = "Column::Object",
		to = "super::object::Column::Internal",
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

impl Related<super::instance::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Instances.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}



#[allow(clippy::large_enum_variant)] // tombstone is an outlier, not the norm! this is a beefy enum
#[derive(Debug, Clone)]
pub enum Event {
	Tombstone,
	Activity(crate::model::activity::Model),
	StrayObject {
		object: crate::model::object::Model,
		liked: Option<String>,
	},
	DeepActivity {
		activity: crate::model::activity::Model,
		object: crate::model::object::Model,
		liked: Option<String>,
	}
}


impl Event {
	pub fn id(&self) -> &str {
		match self {
			Event::Tombstone => "",
			Event::Activity(x) => x.id.as_str(),
			Event::StrayObject { object, liked: _ } => object.id.as_str(),
			Event::DeepActivity { activity: _, liked: _, object } => object.id.as_str(),
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
			Event::DeepActivity { activity, object, liked } =>
				activity.ap().set_object(apb::Node::object(
					object.ap()
						.set_attachment(attachment)
						.set_liked_by_me(if liked.is_some() { Some(true) } else { None })
				)),
			Event::StrayObject { object, liked } => serde_json::Value::new_object()
				.set_activity_type(Some(apb::ActivityType::Activity))
				.set_object(apb::Node::object(
					object.ap()
						.set_attachment(attachment)
						.set_liked_by_me(if liked.is_some() { Some(true) } else { None })
				)),
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
		let liked = res.try_get(crate::model::like::Entity.table_name(), &crate::model::like::Column::Actor.to_string()).ok();
		match (activity, object) {
			(Some(activity), Some(object)) => Ok(Self::DeepActivity { activity, object, liked }),
			(Some(activity), None) => Ok(Self::Activity(activity)),
			(None, Some(object)) => Ok(Self::StrayObject { object, liked }),
			(None, None) => Ok(Self::Tombstone),
		}
	}
}


impl Entity {
	pub fn find_addressed(uid: Option<i64>) -> Select<Entity> {
		let mut select = Entity::find()
			.distinct()
			.select_only()
			.join(sea_orm::JoinType::LeftJoin, Relation::Objects.def())
			.join(sea_orm::JoinType::LeftJoin, Relation::Activities.def())
			.filter(
				// TODO ghetto double inner join because i want to filter out tombstones
				Condition::any()
					.add(crate::model::activity::Column::Id.is_not_null())
					.add(crate::model::object::Column::Id.is_not_null())
			)
			.order_by(Column::Published, Order::Desc);

		if let Some(uid) = uid {
			select = select
				.join(
					sea_orm::JoinType::LeftJoin,
					crate::model::object::Relation::Likes.def()
						.on_condition(move |_l, _r| crate::model::like::Column::Actor.eq(uid).into_condition()),
				)
				.select_column_as(crate::model::like::Column::Actor, format!("{}{}", crate::model::like::Entity.table_name(), crate::model::like::Column::Actor.to_string()));
		}

		for col in crate::model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::object::Entity.table_name(), col.to_string()));
		}

		for col in crate::model::activity::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::activity::Entity.table_name(), col.to_string()));
		}

		select
	}
}
