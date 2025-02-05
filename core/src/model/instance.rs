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
	#[sea_orm(has_many = "super::downtime::Entity")]
	Downtime,
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

impl Related<super::downtime::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Downtime.def()
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
			Ok(res) => {
				// oh my god gotosocial, the face of telling me that
				//  * ah-hoc interop is not needed because "AP is a protocol and the whole point is not working around edge cases"
				//  * you don't consider other AP instances fetching your nodeinfo "crawling"
				// and then just treat us all like crawlers
				// this isn't about user privacy, nodeinfo has that builtin with NULL. it JUST WORKS without workarounds
				// this is about trolling crawlers
				// you're trolling me too
				// check below another example of "not working around edge cases"
				// at least you gave me a way to not throw all gotosocial instances in the bin, so at least there's that
				let noindex_nofollow: Vec<&str> = res.headers()
					.get_all("X-Robots-Tag")
					.iter()
					.filter_map(|h| h.to_str().ok())
					.filter(|h| *h == "noindex" || *h == "nofollow")
					.collect();
				let gotosocial_is_fucking_with_us = noindex_nofollow.contains(&"noindex") && noindex_nofollow.contains(&"nofollow");

				let mut nodeinfo : NodeInfoOwned = res.json().await?;

				if gotosocial_is_fucking_with_us {
					nodeinfo.usage = nodeinfo::types::Usage {
						users: None,
						local_posts: None,
						local_comments: None,
					};
				}

				Ok(nodeinfo)
			},
			// ughhh pleroma wants with json, key without
			Err(_) => reqwest::get(format!("https://{domain}/nodeinfo/2.0"))
				.await?
				.json()
				.await,
		}
	}
}
