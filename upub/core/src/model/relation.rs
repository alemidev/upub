use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "relations")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	pub follower: i64,
	pub following: i64,
	pub accept: Option<i64>,
	pub activity: i64,
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

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
	// TODO this is 2 queries!!! can it be optimized down to 1?
	pub async fn followers(uid: &str, db: &DatabaseConnection) -> Result<Option<Vec<String>>, DbErr> {
		let Some(internal_id) = super::actor::Entity::ap_to_internal(uid, db).await?
		else {
			return Ok(None);
		};
		let out = Entity::find()
			.join(
				sea_orm::JoinType::InnerJoin,
				Entity::belongs_to(super::actor::Entity)
					.from(Column::Follower)
					.to(super::actor::Column::Internal)
					.into()
			)
			.filter(Column::Accept.is_not_null())
			.filter(Column::Following.eq(internal_id))
			.select_only()
			.select_column(super::actor::Column::Id)
			.into_tuple::<String>()
			.all(db)
			.await?;

		Ok(Some(out))
	}

	// TODO this is 2 queries!!! can it be optimized down to 1?
	pub async fn following(uid: &str, db: &DatabaseConnection) -> Result<Option<Vec<String>>, DbErr> {
		let Some(internal_id) = super::actor::Entity::ap_to_internal(uid, db).await?
		else {
			return Ok(None);
		};
		let out = Entity::find()
			.join(
				sea_orm::JoinType::InnerJoin,
				Entity::belongs_to(super::actor::Entity)
					.from(Column::Following)
					.to(super::actor::Column::Internal)
					.into()
			)
			.filter(Column::Accept.is_not_null())
			.filter(Column::Follower.eq(internal_id))
			.select_only()
			.select_column(super::actor::Column::Id)
			.into_tuple::<String>()
			.all(db)
			.await?;

		Ok(Some(out))
	}

	// TODO this is 3 queries!!! can it be optimized down to 1?
	pub fn is_following(follower: i64, following: i64) -> sea_orm::Selector<sea_orm::SelectGetableTuple<i64>> {
		Entity::find()
			.filter(Column::Accept.is_not_null())
			.filter(Column::Follower.eq(follower))
			.filter(Column::Following.eq(following))
			.select_only()
			.select_column(Column::Internal)
			.into_tuple::<i64>()
	}
}
