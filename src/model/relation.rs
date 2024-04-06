use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "relations")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i64,
	pub follower: String,
	pub following: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
// TODO how to represent this User-to-User relation in sea orm??

impl ActiveModelBehavior for ActiveModel {}
