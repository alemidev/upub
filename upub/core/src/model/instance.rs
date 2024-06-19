use nodeinfo::NodeInfoOwned;
use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "instances")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub domain: String,
	pub name: Option<String>,
	pub software: Option<String>,
	pub version: Option<String>,
	pub icon: Option<String>,
	pub down_since: Option<ChronoDateTimeUtc>,
	pub users: Option<i64>,
	pub posts: Option<i64>,
	pub published: ChronoDateTimeUtc,
	pub updated: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::actor::Entity")]
	Actors,
	#[sea_orm(has_many = "super::addressing::Entity")]
	Addressing,
}

impl Related<super::actor::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Actors.def()
	}
}

impl Related<super::addressing::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Addressing.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
	pub fn find_by_domain(domain: &str) -> Select<Entity> {
		Entity::find().filter(Column::Domain.eq(domain))
	}

	pub async fn domain_to_internal(domain: &str, db: &impl ConnectionTrait) -> Result<Option<i64>, DbErr> {
		Entity::find()
			.filter(Column::Domain.eq(domain))
			.select_only()
			.select_column(Column::Internal)
			.into_tuple::<i64>()
			.one(db)
			.await
	}

	pub async fn nodeinfo(domain: &str) -> reqwest::Result<NodeInfoOwned> {
		match reqwest::get(format!("https://{domain}/nodeinfo/2.0.json")).await {
			Ok(res) => res.json().await,
			// ughhh pleroma wants with json, key without
			Err(_) => reqwest::get(format!("https://{domain}/nodeinfo/2.0.json"))
				.await?
				.json()
				.await,
		}
	}
}
