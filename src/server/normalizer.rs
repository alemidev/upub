use apb::{Node, Base, Object, Document};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use crate::{errors::UpubError, model, server::Context};

use super::fetcher::Fetcher;

#[axum::async_trait]
pub trait Normalizer {
	async fn insert_object(&self, obj: impl apb::Object, server: Option<String>) -> crate::Result<model::object::Model>;
}

#[axum::async_trait]
impl Normalizer for super::Context {
	async fn insert_object(&self, object_node: impl apb::Object, server: Option<String>) -> crate::Result<model::object::Model> {
		let mut object_model = model::object::Model::new(&object_node)?;
		let oid = object_model.id.clone();
		let uid = object_model.attributed_to.clone();
		if let Some(server) = server {
			// make sure we're allowed to create this object
			if let Some(object_author) = &object_model.attributed_to {
				if server != Context::server(object_author) {
					return Err(UpubError::forbidden());
				}
			} else if server != Context::server(&object_model.id) {
				return Err(UpubError::forbidden());
			};
		}

		// make sure content only contains a safe subset of html
		if let Some(content) = object_model.content {
			object_model.content = Some(mdhtml::safe_markdown(&content));
		}

		// fix context also for remote posts
		// TODO this is not really appropriate because we're mirroring incorrectly remote objects, but
		// it makes it SOO MUCH EASIER for us to fetch threads and stuff, so we're filling it for them
		match (&object_model.in_reply_to, &object_model.context) {
			(Some(reply_id), None) => // get context from replied object
				object_model.context = self.fetch_object(reply_id).await?.context,
			(None, None) => // generate a new context
				object_model.context = Some(crate::url!(self, "/context/{}", uuid::Uuid::new_v4().to_string())),
			(_, Some(_)) => {}, // leave it as set by user
		}

		// update replies counter
		if let Some(ref in_reply_to) = object_model.in_reply_to {
			if self.fetch_object(in_reply_to).await.is_ok() {
				model::object::Entity::update_many()
					.filter(model::object::Column::Id.eq(in_reply_to))
					.col_expr(model::object::Column::Comments, Expr::col(model::object::Column::Comments).add(1))
					.exec(self.db())
					.await?;
			}
		}
		// update statuses counter
		if let Some(object_author) = uid {
			model::user::Entity::update_many()
				.col_expr(model::user::Column::StatusesCount, Expr::col(model::user::Column::StatusesCount).add(1))
				.filter(model::user::Column::Id.eq(&object_author))
				.exec(self.db())
				.await?;
		}

		model::object::Entity::insert(object_model.clone().into_active_model()).exec(self.db()).await?;

		for attachment in object_node.attachment().flat() {
			let attachment_model = match attachment {
				Node::Empty => continue,
				Node::Array(_) => {
					tracing::warn!("ignoring array-in-array while processing attachments");
					continue
				},
				Node::Link(l) => model::attachment::ActiveModel {
					id: sea_orm::ActiveValue::NotSet,
					url: Set(l.href().to_string()),
					object: Set(oid.clone()),
					document_type: Set(apb::DocumentType::Page),
					name: Set(l.link_name().map(|x| x.to_string())),
					media_type: Set(l.link_media_type().unwrap_or("link").to_string()),
					created: Set(chrono::Utc::now()),
				},
				Node::Object(o) => model::attachment::ActiveModel {
					id: sea_orm::ActiveValue::NotSet,
					url: Set(o.url().id().unwrap_or_else(|| o.id().map(|x| x.to_string()).unwrap_or_default())),
					object: Set(oid.clone()),
					document_type: Set(o.as_document().map_or(apb::DocumentType::Document, |x| x.document_type().unwrap_or(apb::DocumentType::Page))),
					name: Set(o.name().map(|x| x.to_string())),
					media_type: Set(o.media_type().unwrap_or("link").to_string()),
					created: Set(o.published().unwrap_or_else(chrono::Utc::now)),
				},
			};
			model::attachment::Entity::insert(attachment_model)
				.exec(self.db())
				.await?;
		}
		// lemmy sends us an image field in posts, treat it like an attachment i'd say
		if let Some(img) = object_node.image().get() {
			// TODO lemmy doesnt tell us the media type but we use it to display the thing...
			let img_url = img.url().id().unwrap_or_default();
			let media_type = if img_url.ends_with("png") {
				Some("image/png".to_string())
			} else if img_url.ends_with("webp") {
				Some("image/webp".to_string())
			} else if img_url.ends_with("jpeg") || img_url.ends_with("jpg") {
				Some("image/jpeg".to_string())
			} else {
				None
			};

			let attachment_model = model::attachment::ActiveModel {
				id: sea_orm::ActiveValue::NotSet,
				url: Set(img.url().id().unwrap_or_else(|| img.id().map(|x| x.to_string()).unwrap_or_default())),
				object: Set(oid.clone()),
				document_type: Set(img.as_document().map_or(apb::DocumentType::Document, |x| x.document_type().unwrap_or(apb::DocumentType::Page))),
				name: Set(img.name().map(|x| x.to_string())),
				media_type: Set(img.media_type().unwrap_or(media_type.as_deref().unwrap_or("link")).to_string()),
				created: Set(img.published().unwrap_or_else(chrono::Utc::now)),
			};
			model::attachment::Entity::insert(attachment_model)
				.exec(self.db())
				.await?;
		}

		Ok(object_model)
	}
}
