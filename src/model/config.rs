use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "configs")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: String,
	pub accept_follow_requests: bool,
	pub show_followers_count: bool,
	pub show_following_count: bool,
	pub show_followers: bool,
	pub show_following: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::user::Entity",
		from = "Column::Id",
		to = "super::user::Column::Id"
	)]
	User,
}

impl Related<super::user::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
