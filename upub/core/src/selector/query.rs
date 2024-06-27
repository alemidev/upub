use sea_orm::{sea_query::{IntoColumnRef, IntoCondition}, ActiveValue::{NotSet, Set}, ColumnTrait, Condition, EntityName, EntityTrait, Iden, Insert, Iterable, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select, SelectColumns};
use crate::model;

pub struct Query;

impl Query {
	pub fn feed(my_id: Option<i64>) -> Select<model::addressing::Entity> {
		let mut select = model::addressing::Entity::find()
			.distinct_on([
				(model::addressing::Entity, model::addressing::Column::Published).into_column_ref(),
				(model::activity::Entity, model::activity::Column::Internal).into_column_ref(),
			])
			.join(sea_orm::JoinType::LeftJoin, model::addressing::Relation::Activities.def())
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

	pub fn related(from: Option<i64>, to: Option<i64>, pending: bool) -> Select<model::relation::Entity> {
		let mut condition = Condition::all();

		if let Some(from) = from {
			condition = condition.add(model::relation::Column::Follower.eq(from));
		}

		if let Some(to) = to {
			condition = condition.add(model::relation::Column::Following.eq(to));
		}

		if !pending {
			condition = condition.add(model::relation::Column::Accept.is_not_null());
		}

		let direction = match (from, to) {
			// TODO its super arbitrary to pick "Following" as default direction!!!
			(Some(_), Some(_)) => model::relation::Column::Following,
			(None, None) => model::relation::Column::Following,
			// TODO i should really probably change this function's api, maybe add another param??
			(Some(_), None) => model::relation::Column::Following,
			(None, Some(_)) => model::relation::Column::Follower,
		};

		let mut select = model::relation::Entity::find()
			.join(
				sea_orm::JoinType::InnerJoin,
				model::relation::Entity::belongs_to(model::actor::Entity)
					.from(direction)
					.to(model::actor::Column::Internal)
					.into()
			)
			.filter(condition)
			.select_only();

		for column in model::actor::Column::iter() {
			select = select.select_column(column);
		}

		select
	}

	pub fn notify(activity: i64, actor: i64) -> Insert<model::notification::ActiveModel> {
		model::notification::Entity::insert(
			model::notification::ActiveModel {
				internal: NotSet,
				activity: Set(activity),
				actor: Set(actor),
				seen: Set(false),
				published: Set(chrono::Utc::now()),
			}
		)
	}
}
