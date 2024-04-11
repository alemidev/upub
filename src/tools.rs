// yanked from https://github.com/SeaQL/sea-orm/discussions/1502
use sea_orm::{prelude::*, FromQueryResult};
use sea_orm::sea_query::{Alias, IntoIden, SelectExpr, SelectStatement};
use sea_orm::{EntityTrait, QueryTrait};

pub struct Prefixer<S: QueryTrait<QueryStatement = SelectStatement>> {
	pub selector: S,
}

impl<S: QueryTrait<QueryStatement = SelectStatement>> Prefixer<S> {
	pub fn new(selector: S) -> Self {
		Self { selector }
	}
	pub fn add_columns<T: EntityTrait>(mut self, entity: T) -> Self {
		for col in <T::Column as sea_orm::entity::Iterable>::iter() {
			let alias = format!("{}{}", entity.table_name(), col.to_string()); // we use entity.table_name() as prefix
			self.selector.query().expr(SelectExpr {
				expr: col.select_as(col.into_expr()),
				alias: Some(Alias::new(&alias).into_iden()),
				window: None,
			});
		}
		self
	}
}

// adapted from https://github.com/SeaQL/sea-orm/discussions/1502
#[derive(Debug)]
pub struct ActivityWithObject {
	pub activity: crate::model::activity::Model,
	pub object: Option<crate::model::object::Model>,
}

impl FromQueryResult for ActivityWithObject {
	fn from_query_result(res: &sea_orm::QueryResult, _pre: &str) -> Result<Self, sea_orm::DbErr> {
		let activity = crate::model::activity::Model::from_query_result(res, crate::model::activity::Entity.table_name())?;
		let object = crate::model::object::Model::from_query_result(res, crate::model::object::Entity.table_name()).ok();

		Ok(Self { activity, object })
	}
}
