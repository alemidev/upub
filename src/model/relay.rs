use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "relays")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: String,
	pub accepted: bool,
	pub forwarding: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
// TODO how to represent this User-to-User relation in sea orm??

impl ActiveModelBehavior for ActiveModel {}
