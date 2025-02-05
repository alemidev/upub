use sea_orm::{entity::prelude::*, QuerySelect, SelectColumns};

use apb::{ActorMut, ActorType, BaseMut, DocumentMut, EndpointsMut, ObjectMut, PublicKeyMut};

use crate::ext::{JsonVec, TypeName};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Field {
	#[serde(default)]
	pub name: String,
	#[serde(default)]
	pub value: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub verified_at: Option<ChronoDateTimeUtc>,
	#[serde(default, rename = "type")]
	pub field_type: String,
}

impl TypeName for Field {
	fn type_name() -> String {
		"Field".to_string()
	}
}

impl<T: apb::Object> From<T> for Field {
	fn from(value: T) -> Self {
		Field {
			name: value.name().unwrap_or_default().to_string(),
			value: mdhtml::safe_html(&value.value().unwrap_or_default()),
			field_type: "PropertyValue".to_string(), // TODO can we try parsing this instead??
			verified_at: None, // TODO where does verified_at come from? extend apb maybe
		}
	}
}

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
	pub fields: JsonVec<Field>,
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
	pub also_known_as: JsonVec<String>,
	pub moved_to: Option<String>,
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
	#[sea_orm(
		belongs_to = "super::instance::Entity",
		from = "Column::Domain",
		to = "super::instance::Column::Domain",
		on_update = "Cascade",
		on_delete = "NoAction"
	)]
	Instances,
	#[sea_orm(has_many = "super::dislike::Entity")]
	Dislikes,
	#[sea_orm(has_many = "super::like::Entity")]
	Likes,
	#[sea_orm(has_many = "super::mention::Entity")]
	Mentions,
	#[sea_orm(has_many = "super::notification::Entity")]
	Notifications,
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

impl Related<super::instance::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Instances.def()
	}
}

impl Related<super::dislike::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Dislikes.def()
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

impl Related<super::notification::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Notifications.def()
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

	pub async fn ap_to_internal(id: &str, db: &impl ConnectionTrait) -> Result<Option<i64>, DbErr> {
		Entity::find()
			.filter(Column::Id.eq(id))
			.select_only()
			.select_column(Column::Internal)
			.into_tuple::<i64>()
			.one(db)
			.await
	}
}

impl crate::ext::IntoActivityPub for Model {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value {
		let is_local = ctx.is_local(&self.id);
		let id = ctx.id(&self.id);
		apb::new()
			.set_id(Some(self.id.clone()))
			.set_actor_type(Some(self.actor_type))
			.set_name(self.name)
			.set_summary(self.summary)
			.set_icon(apb::Node::maybe_object(self.icon.map(|i|
				apb::new()
					.set_document_type(Some(apb::DocumentType::Image))
					.set_url(apb::Node::link(i.clone()))
			)))
			.set_image(apb::Node::maybe_object(self.image.map(|i|
				apb::new()
					.set_document_type(Some(apb::DocumentType::Image))
					.set_url(apb::Node::link(i.clone()))
			)))
			.set_attachment(apb::Node::array(
				self.fields.0
					.into_iter()
					.filter_map(|x| serde_json::to_value(x).ok())
					.collect()
			))
			.set_published(Some(self.published))
			.set_updated(if self.updated != self.published { Some(self.updated) } else { None })
			.set_preferred_username(Some(self.preferred_username))
			.set_statuses_count(Some(self.statuses_count as u64))
			// local users may want to hide these! default to hidden, and downstream we can opt-in to
			// showing them. for remote users we assume the number is already "protected" by remote
			// instance so we just show it
			.set_followers_count(if is_local { None } else { Some(self.followers_count as u64) })
			.set_following_count(if is_local { None } else { Some(self.following_count as u64) })
			.set_inbox(if is_local { apb::Node::link(crate::url!(ctx, "/actors/{id}/inbox")) } else { apb::Node::maybe_link(self.inbox) })
			.set_outbox(if is_local { apb::Node::link(crate::url!(ctx, "/actors/{id}/outbox")) } else { apb::Node::maybe_link(self.outbox) })
			.set_following(if is_local { apb::Node::link(crate::url!(ctx, "/actors{id}/following")) } else { apb::Node::maybe_link(self.following) })
			.set_followers(if is_local { apb::Node::link(crate::url!(ctx, "/actors/{id}/followers")) } else { apb::Node::maybe_link(self.followers) })
			.set_public_key(apb::Node::object(
				apb::new()
					.set_id(Some(format!("{}#main-key", self.id)))
					.set_owner(Some(self.id))
					.set_public_key_pem(self.public_key)
			))
			.set_endpoints(apb::Node::object(
				apb::new()
					.set_shared_inbox(if is_local { Some(crate::url!(ctx, "/inbox")) } else { self.shared_inbox })
					.set_proxy_url(if is_local { Some(crate::url!(ctx, "/fetch")) } else { None })
			))
			.set_also_known_as(apb::Node::links(self.also_known_as.0))
			.set_moved_to(apb::Node::maybe_link(self.moved_to))
			.set_discoverable(Some(true))
	}
}
