use sea_orm::{sea_query::{IntoColumnRef, IntoCondition}, ColumnTrait, Condition, EntityName, EntityTrait, Iden, Iterable, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select, SelectColumns};
use crate::model;

pub struct Query;

impl Query {
	pub fn activities(my_id: Option<i64>) -> Select<model::addressing::Entity> {
		let mut select = model::addressing::Entity::find()
			.distinct_on([
				(model::addressing::Entity, model::addressing::Column::Published).into_column_ref(),
				(model::activity::Entity, model::activity::Column::Internal).into_column_ref(),
			])
			.join(sea_orm::JoinType::InnerJoin, model::addressing::Relation::Activities.def())
			.join(sea_orm::JoinType::LeftJoin, model::addressing::Relation::Objects.def())
			.filter(
				// TODO ghetto double inner join because i want to filter out tombstones
				Condition::any()
					.add(model::activity::Column::Id.is_not_null())
					.add(model::object::Column::Id.is_not_null())
			)
			.order_by(model::addressing::Column::Published, Order::Desc)
			.order_by(model::activity::Column::Internal, Order::Desc)
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
			.distinct_on([
				(model::addressing::Entity, model::addressing::Column::Published).into_column_ref(),
				(model::object::Entity, model::object::Column::Internal).into_column_ref(),
			])
			.join(sea_orm::JoinType::InnerJoin, model::addressing::Relation::Objects.def())
			.order_by(model::addressing::Column::Published, Order::Desc)
			.order_by(model::object::Column::Internal, Order::Desc)
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
