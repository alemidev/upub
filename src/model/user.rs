use sea_orm::entity::prelude::*;
use crate::activitystream::key::PublicKey as _;

use crate::{activitypub, activitystream::object::{collection::Collection, actor::{Actor, ActorType}}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: String,
	pub domain: String,
	pub actor_type: ActorType,
	pub preferred_username: String,

	pub name: Option<String>,
	pub summary: Option<String>,
	pub image: Option<String>,
	pub icon: Option<String>,

	pub inbox: Option<String>,
	pub shared_inbox: Option<String>,
	pub outbox: Option<String>,
	pub following: Option<String>,
	pub followers: Option<String>,

	pub following_count: i64,
	pub followers_count: i64,

	pub public_key: String,
	pub private_key: Option<String>,

	pub created: ChronoDateTimeUtc,
	pub updated: ChronoDateTimeUtc,
	
	// TODO these are also suggested
	// pub liked: Option<String>,
	// pub streams: Option<String>,
}

impl Model {
	pub fn new(object: &impl Actor) -> Result<Self, super::FieldError> {
		let ap_id = object.id().ok_or(super::FieldError("id"))?.to_string();
		let (domain, preferred_username) = activitypub::split_id(&ap_id);
		Ok(Model {
			id: ap_id, preferred_username, domain,
			actor_type: object.actor_type().ok_or(super::FieldError("type"))?,
			name: object.name().map(|x| x.to_string()),
			summary: object.summary().map(|x| x.to_string()),
			icon: object.icon().id().map(|x| x.to_string()),
			image: object.image().id().map(|x| x.to_string()),
			inbox: object.inbox().id().map(|x| x.to_string()),
			outbox: object.inbox().id().map(|x| x.to_string()),
			shared_inbox: None, // TODO!!! parse endpoints
			followers: object.followers().id().map(|x| x.to_string()),
			following: object.following().id().map(|x| x.to_string()),
			created: object.published().unwrap_or(chrono::Utc::now()),
			updated: chrono::Utc::now(),
			following_count: object.following().get().map(|f| f.total_items().unwrap_or(0)).unwrap_or(0) as i64,
			followers_count: object.followers().get().map(|f| f.total_items().unwrap_or(0)).unwrap_or(0) as i64,
			public_key: object.public_key().get().ok_or(super::FieldError("publicKey"))?.public_key_pem().to_string(),
			private_key: None, // there's no way to transport privkey over AP json, must come from DB
		})
	}
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::activity::Entity")]
	Activity,

	#[sea_orm(has_many = "super::object::Entity")]
	Object,

	#[sea_orm(has_one = "super::config::Entity")]
	Config,

	#[sea_orm(has_one = "super::credential::Entity")]
	Credential,

	#[sea_orm(has_many = "super::session::Entity")]
	Session,

	#[sea_orm(has_many = "super::addressing::Entity")]
	Addressing,
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Activity.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Object.def()
	}
}

impl Related<super::config::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Config.def()
	}
}

impl Related<super::credential::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Credential.def()
	}
}

impl Related<super::session::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Session.def()
	}
}

impl Related<super::addressing::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Addressing.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
