use sea_orm::{entity::prelude::*, sea_query::Alias, QuerySelect, SelectColumns};

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
	ActivitiesAccept,
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Activity",
		to = "super::activity::Column::Id",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	ActivitiesFollow,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Follower",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	ActorsFollower,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Following",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	ActorsFollowing,
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
	pub fn find_followers(id: &str) -> Select<Entity> {
		Entity::find()
			.inner_join(Relation::ActorsFollowing.def())
			.filter(super::actor::Column::ApId.eq(id))
			.left_join(Relation::ActorsFollower.def())
			.select_only()
			.select_column(super::actor::Column::ApId)
			.into_tuple::<String>()
	}
}
