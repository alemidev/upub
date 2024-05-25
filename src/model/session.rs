use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub actor: i32,
	pub secret: String,
	pub expires: ChronoDateTimeUtc,
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

impl Entity {
	pub fn find_by_secret(secret: &str) -> Select<Entity> {
		Entity::find().filter(Column::Secret.eq(secret))
	}
}
