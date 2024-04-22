use apb::{ActivityMut, ObjectMut};
use sea_orm::{entity::prelude::*, FromQueryResult, Iterable, Order, QueryOrder, QuerySelect, SelectColumns};

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





#[derive(Debug)]
pub struct EmbeddedActivity {
	pub activity: crate::model::activity::Model,
	pub object: Option<crate::model::object::Model>,
}

impl EmbeddedActivity {
	pub async fn ap_filled(self, db: &DatabaseConnection) -> crate::Result<serde_json::Value> {
		let a = self.activity.ap();
		match self.object {
			None => Ok(a),
			Some(o) => {
				let attachments = o.find_related(crate::model::attachment::Entity)
					.all(db)
					.await?
					.into_iter()
					.map(|x| x.ap())
					.collect();
				Ok(a.set_object(
					apb::Node::object(o.ap().set_attachment(apb::Node::array(attachments)))
				))
			}
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

#[derive(Debug)]
pub struct WrappedObject {
	pub activity: Option<crate::model::activity::Model>,
	pub object: crate::model::object::Model,
}


impl WrappedObject {
	pub async fn ap_filled(self, db: &DatabaseConnection) -> crate::Result<serde_json::Value> {
		let attachments = self.object.find_related(crate::model::attachment::Entity)
			.all(db)
			.await?
			.into_iter()
			.map(|x| x.ap())
			.collect();
		let o = self.object.ap()
			.set_attachment(apb::Node::Array(attachments));
		match self.activity {
			None => Ok(o),
			Some(a) => Ok(a.ap().set_object(apb::Node::object(o))),
		}
	}
}

impl FromQueryResult for WrappedObject {
	fn from_query_result(res: &sea_orm::QueryResult, _pre: &str) -> Result<Self, sea_orm::DbErr> {
		let activity = crate::model::activity::Model::from_query_result(res, crate::model::activity::Entity.table_name()).ok();
		let object = crate::model::object::Model::from_query_result(res, crate::model::object::Entity.table_name())?;
		Ok(Self { activity, object })
	}
}





impl Entity {
	pub fn find_activities() -> Select<Entity> {
		let mut select = Entity::find()
			.distinct()
			.select_only()
			.join(sea_orm::JoinType::InnerJoin, Relation::Activity.def())
			.join(sea_orm::JoinType::LeftJoin, crate::model::activity::Relation::Object.def())
			.order_by(crate::model::activity::Column::Published, Order::Desc);

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
			.join(sea_orm::JoinType::InnerJoin, Relation::Object.def())
			.join(sea_orm::JoinType::LeftJoin, crate::model::object::Relation::Activity.def())
			.order_by(crate::model::object::Column::Published, Order::Desc);

		for col in crate::model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::object::Entity.table_name(), col.to_string()));
		}

		for col in crate::model::activity::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", crate::model::activity::Entity.table_name(), col.to_string()));
		}

		select
	}
}
