use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "configs")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub actor: String,
	pub accept_follow_requests: bool,
	pub show_followers_count: bool,
	pub show_following_count: bool,
	pub show_followers: bool,
	pub show_following: bool,
}

impl Default for Model {
	fn default() -> Self {
		Model {
			internal: 0, actor: "".into(),
			accept_follow_requests: true,
			show_following_count: true,
			show_following: true,
			show_followers_count: true,
			show_followers: true,
		}
	}
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::actor::Entity",
		from = "Column::Actor",
		to = "super::actor::Column::Id",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	Actors,
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
