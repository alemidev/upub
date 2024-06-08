use apb::{ActivityMut, ObjectMut};
use sea_orm::{sea_query::IntoCondition, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityName, EntityTrait, FromQueryResult, Iden, Iterable, LoaderTrait, ModelTrait, Order, QueryFilter, QueryOrder, QueryResult, QuerySelect, RelationTrait, Select, SelectColumns};

use crate::model;

pub struct Query;

impl Query {
	pub fn activities(my_id: Option<i64>) -> Select<model::addressing::Entity> {
		let mut select = model::addressing::Entity::find()
			// .distinct()
			.join(sea_orm::JoinType::InnerJoin, model::addressing::Relation::Activities.def())
			.join(sea_orm::JoinType::LeftJoin, model::addressing::Relation::Objects.def())
			.filter(
				// TODO ghetto double inner join because i want to filter out tombstones
				Condition::any()
					.add(model::activity::Column::Id.is_not_null())
					.add(model::object::Column::Id.is_not_null())
			)
			.order_by(model::addressing::Column::Published, Order::Desc)
			.select_only();

		for col in model::activity::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", model::activity::Entity.table_name(), col.to_string()));
		}

		for col in model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", model::object::Entity.table_name(), col.to_string()));
		}

		if let Some(uid) = my_id {
			select = select
				.join(
					sea_orm::JoinType::LeftJoin,
					model::object::Relation::Likes.def()
						.on_condition(move |_l, _r| model::like::Column::Actor.eq(uid).into_condition()),
				)
				.select_column_as(model::like::Column::Actor, format!("{}{}", model::like::Entity.table_name(), model::like::Column::Actor.to_string()));
		}

		select
	}

	pub fn objects(my_id: Option<i64>) -> Select<model::addressing::Entity> {
		let mut select = model::addressing::Entity::find()
			// .distinct()
			.join(sea_orm::JoinType::InnerJoin, model::addressing::Relation::Objects.def())
			.order_by(model::addressing::Column::Published, Order::Desc)
			.select_only();

		for col in model::object::Column::iter() {
			select = select.select_column_as(col, format!("{}{}", model::object::Entity.table_name(), col.to_string()));
		}

		if let Some(uid) = my_id {
			select = select
				.join(
					sea_orm::JoinType::LeftJoin,
					model::object::Relation::Likes.def()
						.on_condition(move |_l, _r| model::like::Column::Actor.eq(uid).into_condition()),
				)
				.select_column_as(model::like::Column::Actor, format!("{}{}", model::like::Entity.table_name(), model::like::Column::Actor.to_string()));
		}

		select
	}
}



#[derive(Debug, Clone)]
pub struct EmbeddedActivity {
	pub activity: model::activity::Model,
	pub object: model::object::Model,
	pub liked: Option<i64>,
}

pub struct RichActivity {
	pub activity: model::activity::Model,
	pub object: Option<model::object::Model>,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<model::attachment::Model>>,
}

impl FromQueryResult for RichActivity {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichActivity {
			activity: model::activity::Model::from_query_result(res, model::activity::Entity.table_name())?,
			object: model::object::Model::from_query_result(res, model::object::Entity.table_name()).ok(),
			liked: res.try_get(model::like::Entity.table_name(), &model::like::Column::Actor.to_string()).ok(),
			attachments: None,
		})
	}
}

impl RichActivity {
	pub fn ap(self) -> serde_json::Value {
		self.activity.ap()
			.set_object(apb::Node::maybe_object(
				self.object.map(|x| x.ap()
					.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
					.set_attachment(match self.attachments {
						None => apb::Node::Empty,
						Some(vec) => apb::Node::array(
							vec.into_iter().map(|x| x.ap()).collect()
						),
					})
				)
			))
	}
}

pub struct RichObject {
	pub object: model::object::Model,
	pub liked: Option<i64>,
	pub attachments: Option<Vec<model::attachment::Model>>,
}

impl FromQueryResult for RichObject {
	fn from_query_result(res: &QueryResult, _pre: &str) -> Result<Self, DbErr> {
		Ok(RichObject {
			object: model::object::Model::from_query_result(res, model::object::Entity.table_name())?,
			liked: res.try_get(model::like::Entity.table_name(), &model::like::Column::Actor.to_string()).ok(),
			attachments: None,
		})
	}
}

impl RichObject {
	pub fn ap(self) -> serde_json::Value {
		self.object.ap()
			.set_liked_by_me(if self.liked.is_some() { Some(true) } else { None })
	}
}

#[async_trait::async_trait]
pub trait BatchFillable: Sized {
	async fn with_attachments(self, tx: &impl ConnectionTrait) -> Result<Self, DbErr>;
}

#[async_trait::async_trait]
impl BatchFillable for Vec<RichActivity> {
	// TODO 3 iterations... can we make it in less passes?
	async fn with_attachments(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr> {
		let objects : Vec<model::object::Model> = self
			.iter()
			.filter_map(|x| x.object.as_ref().cloned())
			.collect();

		let attachments = objects.load_many(model::attachment::Entity, tx).await?;

		let mut out : std::collections::BTreeMap<i64, Vec<model::attachment::Model>> = std::collections::BTreeMap::new();
		for attach in attachments.into_iter().flatten() {
			match out.entry(attach.object) {
				std::collections::btree_map::Entry::Vacant(a) => { a.insert(vec![attach]); },
				std::collections::btree_map::Entry::Occupied(mut e) => { e.get_mut().push(attach); },
			}
		}

		for activity in self.iter_mut() {
			if let Some(ref object) = activity.object {
				activity.attachments = out.remove(&object.internal);
			}
		}

		Ok(self)
	}
}

#[async_trait::async_trait]
impl BatchFillable for Vec<RichObject> {
	// TODO 3 iterations... can we make it in less passes?
	async fn with_attachments(mut self, db: &impl ConnectionTrait) -> Result<Self, DbErr> {
		let objects : Vec<model::object::Model> = self
			.iter()
			.map(|o| o.object.clone())
			.collect();

		let attachments = objects.load_many(model::attachment::Entity, db).await?;

		let mut out : std::collections::BTreeMap<i64, Vec<model::attachment::Model>> = std::collections::BTreeMap::new();
		for attach in attachments.into_iter().flatten() {
			match out.entry(attach.object) {
				std::collections::btree_map::Entry::Vacant(a) => { a.insert(vec![attach]); },
				std::collections::btree_map::Entry::Occupied(mut e) => { e.get_mut().push(attach); },
			}
		}

		for obj in self.iter_mut() {
			obj.attachments = out.remove(&obj.object.internal);
		}

		Ok(self)
	}
}

#[async_trait::async_trait]
impl BatchFillable for RichActivity {
	async fn with_attachments(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr> {
		if let Some(ref obj) = self.object {
			self.attachments = Some(
				obj.find_related(model::attachment::Entity)
					.all(tx)
					.await?
			);
		}

		Ok(self)
	}
}

#[async_trait::async_trait]
impl BatchFillable for RichObject {
	async fn with_attachments(mut self, tx: &impl ConnectionTrait) -> Result<Self, DbErr> {
		self.attachments = Some(
			self.object.find_related(model::attachment::Entity)
				.all(tx)
				.await?
		);

		Ok(self)
	}
}
