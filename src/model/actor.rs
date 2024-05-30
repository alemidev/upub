use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

use apb::{Actor, ActorMut, ActorType, BaseMut, DocumentMut, Endpoints, EndpointsMut, Object, ObjectMut, PublicKey, PublicKeyMut};

use crate::{errors::UpubError, routes::activitypub::jsonld::LD};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "actors")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub internal: i64,
	#[sea_orm(unique)]
	pub id: String,
	pub actor_type: ActorType,
	pub domain: String,
	pub name: Option<String>,
	pub summary: Option<String>,
	pub image: Option<String>,
	pub icon: Option<String>,
	pub preferred_username: String,
	pub inbox: Option<String>,
	pub shared_inbox: Option<String>,
	pub outbox: Option<String>,
	pub following: Option<String>,
	pub followers: Option<String>,
	pub following_count: i32,
	pub followers_count: i32,
	pub statuses_count: i32,
	pub public_key: String,
	pub private_key: Option<String>,
	pub published: ChronoDateTimeUtc,
	pub updated: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::activity::Entity")]
	Activities,
	#[sea_orm(has_many = "super::addressing::Entity")]
	Addressing,
	#[sea_orm(has_many = "super::announce::Entity")]
	Announces,
	#[sea_orm(has_many = "super::config::Entity")]
	Configs,
	#[sea_orm(has_many = "super::credential::Entity")]
	Credentials,
	#[sea_orm(has_many = "super::delivery::Entity")]
	Deliveries,
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::Domain",
		to = "super::instance::Column::Domain",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Instances,
	#[sea_orm(has_many = "super::like::Entity")]
	Likes,
	#[sea_orm(has_many = "super::mention::Entity")]
	Mentions,
	#[sea_orm(has_many = "super::object::Entity")]
	Objects,
	#[sea_orm(has_many = "super::relation::Entity")]
	Relations,
	#[sea_orm(has_many = "super::session::Entity")]
	Sessions,
}

impl Related<super::activity::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Activities.def()
	}
}

impl Related<super::addressing::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Addressing.def()
	}
}

impl Related<super::announce::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Announces.def()
	}
}

impl Related<super::config::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Configs.def()
	}
}

impl Related<super::credential::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Credentials.def()
	}
}

impl Related<super::delivery::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Deliveries.def()
	}
}

impl Related<super::instance::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Instances.def()
	}
}

impl Related<super::like::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Likes.def()
	}
}

impl Related<super::mention::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Mentions.def()
	}
}

impl Related<super::object::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Objects.def()
	}
}

impl Related<super::relation::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Relations.def()
	}
}

impl Related<super::session::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Sessions.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
	pub fn find_by_ap_id(id: &str) -> Select<Entity> {
		Entity::find().filter(Column::Id.eq(id))
	}

	pub fn delete_by_ap_id(id: &str) -> sea_orm::DeleteMany<Entity> {
		Entity::delete_many().filter(Column::Id.eq(id))
	}

	pub async fn ap_to_internal(id: &str, db: &DatabaseConnection) -> crate::Result<i64> {
		Entity::find()
			.filter(Column::Id.eq(id))
			.select_only()
			.select_column(Column::Internal)
			.into_tuple::<i64>()
			.one(db)
			.await?
			.ok_or_else(UpubError::not_found)
	}
}

impl ActiveModel {
	pub fn new(object: &impl Actor) -> Result<Self, super::FieldError> {
		let ap_id = object.id().ok_or(super::FieldError("id"))?.to_string();
		let (domain, fallback_preferred_username) = split_user_id(&ap_id);
		Ok(ActiveModel {
			internal: sea_orm::ActiveValue::NotSet,
			domain: sea_orm::ActiveValue::Set(domain),
			id: sea_orm::ActiveValue::Set(ap_id),
			preferred_username: sea_orm::ActiveValue::Set(object.preferred_username().unwrap_or(&fallback_preferred_username).to_string()),
			actor_type: sea_orm::ActiveValue::Set(object.actor_type().ok_or(super::FieldError("type"))?),
			name: sea_orm::ActiveValue::Set(object.name().map(|x| x.to_string())),
			summary: sea_orm::ActiveValue::Set(object.summary().map(|x| x.to_string())),
			icon: sea_orm::ActiveValue::Set(object.icon().get().and_then(|x| x.url().id())),
			image: sea_orm::ActiveValue::Set(object.image().get().and_then(|x| x.url().id())),
			inbox: sea_orm::ActiveValue::Set(object.inbox().id()),
			outbox: sea_orm::ActiveValue::Set(object.outbox().id()),
			shared_inbox: sea_orm::ActiveValue::Set(object.endpoints().get().and_then(|x| Some(x.shared_inbox()?.to_string()))),
			followers: sea_orm::ActiveValue::Set(object.followers().id()),
			following: sea_orm::ActiveValue::Set(object.following().id()),
			published: sea_orm::ActiveValue::Set(object.published().unwrap_or(chrono::Utc::now())),
			updated: sea_orm::ActiveValue::Set(chrono::Utc::now()),
			following_count: sea_orm::ActiveValue::Set(object.following_count().unwrap_or(0) as i32),
			followers_count: sea_orm::ActiveValue::Set(object.followers_count().unwrap_or(0) as i32),
			statuses_count: sea_orm::ActiveValue::Set(object.statuses_count().unwrap_or(0) as i32),
			public_key: sea_orm::ActiveValue::Set(object.public_key().get().ok_or(super::FieldError("publicKey"))?.public_key_pem().to_string()),
			private_key: sea_orm::ActiveValue::Set(None), // there's no way to transport privkey over AP json, must come from DB
		})
	}
}

impl Model {
	pub fn ap(self) -> serde_json::Value {
		serde_json::Value::new_object()
			.set_id(Some(&self.id))
			.set_actor_type(Some(self.actor_type))
			.set_name(self.name.as_deref())
			.set_summary(self.summary.as_deref())
			.set_icon(apb::Node::maybe_object(self.icon.map(|i|
				serde_json::Value::new_object()
					.set_document_type(Some(apb::DocumentType::Image))
					.set_url(apb::Node::link(i.clone()))
			)))
			.set_image(apb::Node::maybe_object(self.image.map(|i|
				serde_json::Value::new_object()
					.set_document_type(Some(apb::DocumentType::Image))
					.set_url(apb::Node::link(i.clone()))
			)))
			.set_published(Some(self.published))
			.set_preferred_username(Some(&self.preferred_username))
			.set_statuses_count(Some(self.statuses_count as u64))
			.set_followers_count(Some(self.followers_count as u64))
			.set_following_count(Some(self.following_count as u64))
			.set_inbox(apb::Node::maybe_link(self.inbox))
			.set_outbox(apb::Node::maybe_link(self.outbox))
			.set_following(apb::Node::maybe_link(self.following))
			.set_followers(apb::Node::maybe_link(self.followers))
			.set_public_key(apb::Node::object(
				serde_json::Value::new_object()
					.set_id(Some(&format!("{}#main-key", self.id)))
					.set_owner(Some(&self.id))
					.set_public_key_pem(&self.public_key)
			))
			.set_endpoints(apb::Node::object(
				serde_json::Value::new_object()
					.set_shared_inbox(self.shared_inbox.as_deref())
			))
			.set_discoverable(Some(true))
	}
}

fn split_user_id(id: &str) -> (String, String) {
	let clean = id
		.replace("http://", "")
		.replace("https://", "");
	let mut splits = clean.split('/');
	let first = splits.next().unwrap_or("");
	let last = splits.last().unwrap_or(first);
	(first.to_string(), last.to_string())
}
