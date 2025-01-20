use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "relations")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub follower: i64,
	pub following: i64,
	pub accept: Option<i64>,
	pub activity: i64,
	pub follower_instance: i64,
	pub following_instance: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Accept",
		to = "super::activity::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	ActivitiesAccept,
	#[sea_orm(
		belongs_to = "super::activity::Entity",
		from = "Column::Activity",
		to = "super::activity::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	ActivitiesFollow,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Follower",
		to = "super::actor::Column::Internal",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	ActorsFollower,
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Following",
		to = "super::actor::Column::Internal",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	ActorsFollowing,
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::FollowerInstance",
		to = "super::instance::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	InstancesFollower,
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::FollowingInstance",
		to = "super::instance::Column::Internal",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	InstancesFollowing,
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ActorsFollowing.def()
	}
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::ActivitiesFollow.def()
	}
}

impl Related<super::instance::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::InstancesFollowing.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
