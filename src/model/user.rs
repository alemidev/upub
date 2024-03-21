use sea_orm::entity::prelude::*;
use crate::activitystream::object::document::DocumentType;
use crate::activitystream::prelude::*;

use crate::{activitypub, activitystream::{object::actor::ActorType, BaseType, Node, ObjectType}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
	#[sea_orm(primary_key)]
	/// must be full AP ID, since they are unique over the network
	pub id: String,
	pub domain: String,
	pub actor_type: ActorType,

	pub name: String,
	pub summary: Option<String>,
	pub image: Option<String>,
	pub icon: Option<String>,

	pub preferred_username: Option<String>,
	pub inbox: Option<String>,
	pub shared_inbox: Option<String>,
	pub outbox: Option<String>,
	pub following: Option<String>,
	pub followers: Option<String>,

	pub public_key: String,
	pub private_key: Option<String>,

	pub created: ChronoDateTimeUtc,
	pub updated: ChronoDateTimeUtc,
	
	// TODO these are also suggested
	// pub liked: Option<String>,
	// pub streams: Option<String>,
}

use crate::activitystream::object::{actor::Actor, collection::Collection, document::Image};

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::activity::Entity")]
	Activity,

	#[sea_orm(has_many = "super::object::Entity")]
	Object,
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

impl ActiveModelBehavior for ActiveModel {}

impl crate::activitystream::Base for Model {
	fn id(&self) -> Option<&str> {
		Some(&self.id)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Object(ObjectType::Actor(self.actor_type)))
	}

	fn underlying_json_object(self) -> serde_json::Value {
		crate::activitystream::object()
			.set_id(Some(&self.id))
			.set_actor_type(Some(self.actor_type))
			.set_name(Some(&self.name))
			.set_summary(self.summary.as_deref())
			.set_icon(self.icon())
			.set_image(self.image())
			.set_preferred_username(self.preferred_username.as_deref())
			.set_inbox(self.inbox())
			.set_outbox(self.outbox())
			.set_following(self.following())
			.set_followers(self.followers())
			.set_public_key(self.public_key())
			.set_discoverable(Some(true))
			.set_endpoints(None) // TODO dirty fix to put an empty object
			.underlying_json_object()
	}
}

impl crate::activitystream::object::Object for Model {
	fn name(&self) -> Option<&str> {
		Some(&self.name)
	}

	fn summary(&self) -> Option<&str> {
		self.summary.as_deref()
	}

	fn icon(&self) -> Node<impl Image> {
		match &self.icon {
			Some(x) => Node::object(
				crate::activitystream::raw_object()
					.set_document_type(Some(DocumentType::Image))
					.set_url(Node::link(x.clone()))
			),
			None => Node::Empty,
		}
	}

	fn image(&self) -> Node<impl Image> {
		match &self.image {
			Some(x) => Node::object(
				crate::activitystream::raw_object()
					.set_document_type(Some(DocumentType::Image))
					.set_url(Node::link(x.clone()))
			),
			None => Node::Empty,
		}
	}

	fn published(&self) -> Option<chrono::DateTime<chrono::Utc>> {
		Some(self.created)
	}
}

impl crate::activitystream::object::actor::Actor for Model {
	fn actor_type(&self) -> Option<ActorType> {
		Some(self.actor_type)
	}

	fn preferred_username(&self) -> Option<&str> {
		self.preferred_username.as_deref()
	}

	fn inbox(&self) -> Node<impl Collection> {
		Node::link(self.inbox.clone().unwrap_or(format!("https://{}/users/{}/inbox", self.domain, self.name)))
	}

	fn outbox(&self) -> Node<impl Collection> {
		Node::link(self.outbox.clone().unwrap_or(format!("https://{}/users/{}/outbox", self.domain, self.name)))
	}

	fn following(&self) -> Node<impl Collection> {
		Node::link(self.following.clone().unwrap_or(format!("https://{}/users/{}/following", self.domain, self.name)))
	}

	fn followers(&self) -> Node<impl Collection> {
		Node::link(self.following.clone().unwrap_or(format!("https://{}/users/{}/followers", self.domain, self.name)))
	}

	fn public_key(&self) -> Node<impl crate::activitystream::key::PublicKey> {
		Node::object(
			crate::activitystream::raw_object()
				.set_id(Some(&format!("{}#main-key", self.id))) // TODO is this some standard??
				.set_public_key_pem(&self.public_key)
				.set_owner(Some(&self.id))
		)
	}
}

impl Model {
	pub fn new(object: &impl Actor) -> Result<Self, super::FieldError> {
		let ap_id = object.id().ok_or(super::FieldError("id"))?.to_string();
		let (domain, name) = activitypub::split_id(&ap_id);
		Ok(Model {
			id: ap_id, name, domain,
			actor_type: object.actor_type().ok_or(super::FieldError("type"))?,
			preferred_username: object.preferred_username().map(|x| x.to_string()),
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
			public_key: object.public_key().get().ok_or(super::FieldError("publicKey"))?.public_key_pem().to_string(),
			private_key: None, // there's no way to transport privkey over AP json, must come from DB
		})
	}
}
