use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "relations")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub follower: i32,
	pub following: i32,
	pub accept: Option<i32>,
	pub activity: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Accept",
		to = "super::activity::Column::Id",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Activities2,
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Activity",
		to = "super::activity::Column::Id",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Activities1,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Follower",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Actors2,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Following",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Actors1,
}

impl ActiveModelBehavior for ActiveModel {}
