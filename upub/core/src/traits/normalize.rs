use apb::{field::OptionalString, Collection, Document, Endpoints, Node, Object, PublicKey};
use sea_orm::{sea_query::Expr, ActiveValue::{NotSet, Set}, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter};

use super::Addresser;

#[derive(Debug, thiserror::Error)]
pub enum NormalizerError {
	#[error("normalized document misses required field: {0:?}")]
	Malformed(#[from] apb::FieldErr),

	#[error("wrong object type: expected {0}, got {1}")]
	WrongType(apb::BaseType, apb::BaseType),

	#[error("database error while normalizing object: {0:?}")]
	DbErr(#[from] sea_orm::DbErr),
}

#[async_trait::async_trait]
pub trait Normalizer {
	async fn insert_object(&self, obj: impl apb::Object, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, NormalizerError>;
	async fn insert_activity(&self, act: impl apb::Activity, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, NormalizerError>;
}

#[async_trait::async_trait]
impl Normalizer for crate::Context {

	async fn insert_object(&self, object: impl apb::Object, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, NormalizerError> {
		let mut object_model = AP::object(&object)?;

		// make sure content only contains a safe subset of html
		if let Some(content) = object_model.content {
			object_model.content = Some(mdhtml::safe_html(&content));
		}

		// fix context for remote posts
		// > if any link is broken or we get rate limited, the whole insertion fails which is
		// > kind of dumb. there should be a job system so this can be done in waves. or maybe there's
		// > some whole other way to do this?? im thinking but misskey aaaa!! TODO
		if let Ok(reply) = object.in_reply_to().id() {
			if let Some(o) = crate::model::object::Entity::find_by_ap_id(reply).one(tx).await? {
				object_model.context = o.context;
			} else {
				object_model.context = None; // TODO to be filled by some other task
			}
		} else {
			object_model.context = Some(object_model.id.clone());
		}

		let mut object_active_model = object_model.clone().into_active_model();
		object_active_model.internal = NotSet;
		crate::model::object::Entity::insert(object_active_model).exec(tx).await?;
		object_model.internal = crate::model::object::Entity::ap_to_internal(&object_model.id, tx)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(object_model.id.clone()))?;

		self.address_object(&object_model, tx).await?;

		// update replies counter
		if let Some(ref in_reply_to) = object_model.in_reply_to {
			crate::model::object::Entity::update_many()
				.filter(crate::model::object::Column::Id.eq(in_reply_to))
				.col_expr(crate::model::object::Column::Replies, Expr::col(crate::model::object::Column::Replies).add(1))
				.exec(tx)
				.await?;
		}
		// update statuses counter
		if let Some(ref object_author) = object_model.attributed_to {
			crate::model::actor::Entity::update_many()
				.col_expr(crate::model::actor::Column::StatusesCount, Expr::col(crate::model::actor::Column::StatusesCount).add(1))
				.filter(crate::model::actor::Column::Id.eq(object_author))
				.exec(tx)
				.await?;
		}

		for attachment in object.attachment().flat() {
			let attachment_model = match attachment {
				Node::Empty => continue,
				Node::Array(_) => {
					tracing::warn!("ignoring array-in-array while processing attachments");
					continue
				},
				Node::Link(l) => crate::model::attachment::ActiveModel {
					internal: sea_orm::ActiveValue::NotSet,
					url: Set(l.href().to_string()),
					object: Set(object_model.internal),
					document_type: Set(apb::DocumentType::Page),
					name: Set(l.name().str()),
					media_type: Set(l.media_type().unwrap_or("link").to_string()),
					published: Set(chrono::Utc::now()),
				},
				Node::Object(o) =>
					AP::attachment_q(o.as_document()?, object_model.internal)?,
			};
			crate::model::attachment::Entity::insert(attachment_model)
				.exec(tx)
				.await?;
		}
		// lemmy sends us an image field in posts, treat it like an attachment i'd say
		if let Some(img) = object.image().get() {
			// TODO lemmy doesnt tell us the media type but we use it to display the thing...
			let img_url = img.url().id().str().unwrap_or_default();
			let media_type = if img_url.ends_with("png") {
				Some("image/png".to_string())
			} else if img_url.ends_with("webp") {
				Some("image/webp".to_string())
			} else if img_url.ends_with("jpeg") || img_url.ends_with("jpg") {
				Some("image/jpeg".to_string())
			} else {
				None
			};

			let mut attachment_model = AP::attachment_q(img, object_model.internal)?;

			// ugly fix for lemmy
			if let Some(m) = media_type {
				if img.media_type().ok().is_none() {
					attachment_model.media_type = Set(m);
				}
			}

			crate::model::attachment::Entity::insert(attachment_model)
				.exec(tx)
				.await?;
		}

		Ok(object_model)
	}

	async fn insert_activity(&self, activity: impl apb::Activity, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, NormalizerError> {
		let mut activity_model = AP::activity(&activity)?;

		let mut active_model = activity_model.clone().into_active_model();
		active_model.internal = NotSet;
		crate::model::activity::Entity::insert(active_model)
			.exec(tx)
			.await?;

		let internal = crate::model::activity::Entity::ap_to_internal(&activity_model.id, tx)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(activity_model.id.clone()))?;
		activity_model.internal = internal;

		self.address_activity(&activity_model, tx).await?;

		Ok(activity_model)
	}
}

pub struct AP;

impl AP {
	pub fn activity(activity: &impl apb::Activity) -> Result<crate::model::activity::Model, NormalizerError> {
		let t = activity.base_type()?;
		if !matches!(t, apb::BaseType::Object(apb::ObjectType::Activity(_))) {
			return Err(NormalizerError::WrongType(apb::BaseType::Object(apb::ObjectType::Activity(apb::ActivityType::Activity)), t));
		}
		Ok(crate::model::activity::Model {
			internal: 0,
			id: activity.id()?.to_string(),
			activity_type: activity.activity_type()?,
			actor: activity.actor().id()?.to_string(),
			object: activity.object().id().str(),
			target: activity.target().id().str(),
			published: activity.published().unwrap_or(chrono::Utc::now()),
			to: activity.to().into(),
			bto: activity.bto().into(),
			cc: activity.cc().into(),
			bcc: activity.bcc().into(),
		})
	}

	pub fn activity_q(activity: &impl apb::Activity) -> Result<crate::model::activity::ActiveModel, NormalizerError> {
		let mut m = AP::activity(activity)?.into_active_model();
		m.internal = NotSet;
		Ok(m)
	}




	pub fn attachment(document: &impl apb::Document, parent: i64) -> Result<crate::model::attachment::Model, NormalizerError> {
		let t = document.base_type()?;
		if !matches!(t, apb::BaseType::Object(apb::ObjectType::Document(_))) {
			return Err(NormalizerError::WrongType(apb::BaseType::Object(apb::ObjectType::Document(apb::DocumentType::Document)), t));
		}
		Ok(crate::model::attachment::Model {
			internal: 0,
			url: document.url().id().str().unwrap_or_default(),
			object: parent,
			document_type: document.as_document().map_or(apb::DocumentType::Document, |x| x.document_type().unwrap_or(apb::DocumentType::Page)),
			name: document.name().str(),
			media_type: document.media_type().unwrap_or("link").to_string(),
			published: document.published().unwrap_or_else(|_| chrono::Utc::now()),
		})
	}

	pub fn attachment_q(document: &impl apb::Document, parent: i64) -> Result<crate::model::attachment::ActiveModel, NormalizerError> {
		let mut m = AP::attachment(document, parent)?.into_active_model();
		m.internal = NotSet;
		Ok(m)
	}



	pub fn object(object: &impl apb::Object) -> Result<crate::model::object::Model, NormalizerError> {
		let t = object.base_type()?;
		if !matches!(t,
			apb::BaseType::Object(
				apb::ObjectType::Object
				| apb::ObjectType::Note
				| apb::ObjectType::Article
				| apb::ObjectType::Event
				| apb::ObjectType::Place
				| apb::ObjectType::Profile
				| apb::ObjectType::Document(apb::DocumentType::Page) // why Document lemmy??????
			)
		) {
			return Err(NormalizerError::WrongType(apb::BaseType::Object(apb::ObjectType::Object), t));
		}
		Ok(crate::model::object::Model {
			internal: 0,
			id: object.id()?.to_string(),
			object_type: object.object_type()?,
			attributed_to: object.attributed_to().id().str(),
			name: object.name().str(),
			summary: object.summary().str(),
			content: object.content().str(),
			context: object.context().id().str(),
			in_reply_to: object.in_reply_to().id().str(),
			published: object.published().unwrap_or_else(|_| chrono::Utc::now()),
			updated: object.updated().unwrap_or_else(|_| chrono::Utc::now()),
			url: object.url().id().str(),
			replies: object.replies().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i32,
			likes: object.likes().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i32,
			announces: object.shares().get()
				.map_or(0, |x| x.total_items().unwrap_or(0)) as i32,
			audience: object.audience().id().str(),
			to: object.to().into(),
			bto: object.bto().into(),
			cc: object.cc().into(),
			bcc: object.bcc().into(),

			sensitive: object.sensitive().unwrap_or(false),
		})
	}

	pub fn object_q(object: &impl apb::Object) -> Result<crate::model::object::ActiveModel, NormalizerError> {
		let mut m = AP::object(object)?.into_active_model();
		m.internal = NotSet;
		Ok(m)
	}



	pub fn actor(actor: &impl apb::Actor) -> Result<crate::model::actor::Model, NormalizerError> {
		let t = actor.base_type()?;
		if !matches!(t, apb::BaseType::Object(apb::ObjectType::Actor(_))) {
			return Err(NormalizerError::WrongType(apb::BaseType::Object(apb::ObjectType::Actor(apb::ActorType::Person)), t));
		}
		let ap_id = actor.id()?.to_string();
		let (domain, fallback_preferred_username) = {
			let clean = ap_id
				.replace("http://", "")
				.replace("https://", "");
			let mut splits = clean.split('/');
			let first = splits.next().unwrap_or("");
			let last = splits.last().unwrap_or(first);
			(first.to_string(), last.to_string())
		};
		Ok(crate::model::actor::Model {
			internal: 0,
			domain,
			id: ap_id,
			preferred_username: actor.preferred_username().unwrap_or(&fallback_preferred_username).to_string(),
			actor_type: actor.actor_type()?,
			name: actor.name().str(),
			summary: actor.summary().str(),
			icon: actor.icon().get().and_then(|x| x.url().id().str()),
			image: actor.image().get().and_then(|x| x.url().id().str()),
			inbox: actor.inbox().id().str(),
			outbox: actor.outbox().id().str(),
			shared_inbox: actor.endpoints().get().and_then(|x| x.shared_inbox().str()),
			followers: actor.followers().id().str(),
			following: actor.following().id().str(),
			published: actor.published().unwrap_or(chrono::Utc::now()),
			updated: chrono::Utc::now(),
			following_count: actor.following_count().unwrap_or(0) as i32,
			followers_count: actor.followers_count().unwrap_or(0) as i32,
			statuses_count: actor.statuses_count().unwrap_or(0) as i32,
			public_key: actor.public_key().get().ok_or(apb::FieldErr("publicKey"))?.public_key_pem().to_string(),
			private_key: None, // there's no way to transport privkey over AP json, must come from DB
		})
	}

	pub fn actor_q(actor: &impl apb::Actor) -> Result<crate::model::actor::ActiveModel, NormalizerError> {
		let mut m = AP::actor(actor)?.into_active_model();
		m.internal = NotSet;
		Ok(m)
	}
}
