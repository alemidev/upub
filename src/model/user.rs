use sea_orm::entity::prelude::*;

use apb::{Actor, ActorMut, ActorType, BaseMut, Collection, CollectionMut, DocumentMut, Object, ObjectMut, PublicKey, PublicKeyMut};

use crate::routes::activitypub::jsonld::LD;

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
	pub statuses_count: i64,

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
		let (domain, fallback_preferred_username) = split_user_id(&ap_id);
		Ok(Model {
			id: ap_id,
			domain,
			preferred_username: object.preferred_username().unwrap_or(&fallback_preferred_username).to_string(),
			actor_type: object.actor_type().ok_or(super::FieldError("type"))?,
			name: object.name().map(|x| x.to_string()),
			summary: object.summary().map(|x| x.to_string()),
			icon: object.icon().get().map(|x| x.url().id().unwrap_or_default()),
			image: object.image().get().map(|x| x.url().id().unwrap_or_default()),
			inbox: object.inbox().id(),
			outbox: object.outbox().id(),
			shared_inbox: None, // TODO!!! parse endpoints
			followers: object.followers().id(),
			following: object.following().id(),
			created: object.published().unwrap_or(chrono::Utc::now()),
			updated: chrono::Utc::now(),
			following_count: object.generator().get().map_or(0, |f| f.as_collection().map_or(0, |f| f.total_items().unwrap_or(0))) as i64,
			followers_count: object.audience().get().map_or(0, |f| f.as_collection().map_or(0, |f| f.total_items().unwrap_or(0))) as i64,
			statuses_count: object.replies().get().map_or(0, |o| o.total_items().unwrap_or(0)) as i64,
			public_key: object.public_key().get().ok_or(super::FieldError("publicKey"))?.public_key_pem().to_string(),
			private_key: None, // there's no way to transport privkey over AP json, must come from DB
		})
	}

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
			.set_published(Some(self.created))
			.set_preferred_username(Some(&self.preferred_username))
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
			.set_discoverable(Some(true))
			.set_endpoints(apb::Node::Empty)
			.set_replies(apb::Node::object(
				serde_json::Value::new_object()
					.set_id(self.outbox.as_deref())
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.statuses_count as u64))
			))
			.set_audience(apb::Node::object(
				serde_json::Value::new_object()
					.set_id(self.followers.as_deref())
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.followers_count as u64))
			))
			.set_generator(apb::Node::object(
				serde_json::Value::new_object()
					.set_id(self.following.as_deref())
					.set_collection_type(Some(apb::CollectionType::OrderedCollection))
					.set_total_items(Some(self.following_count as u64))
			))
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

fn split_user_id(id: &str) -> (String, String) {
	let clean = id
		.replace("http://", "")
		.replace("https://", "");
	let mut splits = clean.split('/');
	let first = splits.next().unwrap_or("");
	let last = splits.last().unwrap_or(first);
	(first.to_string(), last.to_string())
}
