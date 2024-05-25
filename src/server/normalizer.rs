use apb::{Node, Base, Object, Document};
use sea_orm::{sea_query::Expr, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use crate::{errors::UpubError, model, server::Context};

use super::fetcher::Fetcher;

#[axum::async_trait]
pub trait Normalizer {
	async fn insert_object(&self, obj: impl apb::Object, server: Option<String>) -> crate::Result<model::object::Model>;
}

#[axum::async_trait]
impl Normalizer for super::Context {
	async fn insert_object(&self, object_node: impl apb::Object, server: Option<String>) -> crate::Result<model::object::Model> {
		let oid = object_node.id().ok_or_else(UpubError::bad_request)?.to_string();
		let uid = object_node.attributed_to().id();
		let mut object_model = model::object::ActiveModel::new(&object_node)?;
		if let Some(server) = server {
			// make sure we're allowed to create this object
			if let Set(Some(object_author)) = &object_model.attributed_to {
				if server != Context::server(object_author) {
					return Err(UpubError::forbidden());
				}
			} else if server != Context::server(&oid) {
				return Err(UpubError::forbidden());
			};
		}

		// make sure content only contains a safe subset of html
		if let Set(Some(content)) = object_model.content {
			object_model.content = Set(Some(mdhtml::safe_html(&content)));
		}

		// fix context for remote posts
		// > note that this will effectively recursively try to fetch the parent object, in order to find
		// > the context (which is id of topmost object). there's a recursion limit of 16 hidden inside
		// > btw! also if any link is broken or we get rate limited, the whole insertion fails which is
		// > kind of dumb. there should be a job system so this can be done in waves. or maybe there's
		// > some whole other way to do this?? im thinking but misskey aaaa!! TODO
		if let Set(Some(ref reply)) = object_model.in_reply_to {
			if let Some(o) = model::object::Entity::find_by_ap_id(reply).one(self.db()).await? {
				object_model.context = Set(o.context);
			} else {
				object_model.context = Set(None); // TODO to be filled by some other task
			}
		} else {
			object_model.context = Set(Some(oid.clone()));
		}

		model::object::Entity::insert(object_model.clone().into_active_model()).exec(self.db()).await?;
		let object = model::object::Entity::find_by_ap_id(&oid).one(self.db()).await?.ok_or_else(UpubError::internal_server_error)?;

		// update replies counter
		if let Set(Some(ref in_reply_to)) = object_model.in_reply_to {
			if self.fetch_object(in_reply_to).await.is_ok() {
				model::object::Entity::update_many()
					.filter(model::object::Column::Id.eq(in_reply_to))
					.col_expr(model::object::Column::Replies, Expr::col(model::object::Column::Replies).add(1))
					.exec(self.db())
					.await?;
			}
		}
		// update statuses counter
		if let Some(object_author) = uid {
			model::actor::Entity::update_many()
				.col_expr(model::actor::Column::StatusesCount, Expr::col(model::actor::Column::StatusesCount).add(1))
				.filter(model::actor::Column::Id.eq(&object_author))
				.exec(self.db())
				.await?;
		}

		for attachment in object_node.attachment().flat() {
			let attachment_model = match attachment {
				Node::Empty => continue,
				Node::Array(_) => {
					tracing::warn!("ignoring array-in-array while processing attachments");
					continue
				},
				Node::Link(l) => model::attachment::ActiveModel {
					internal: sea_orm::ActiveValue::NotSet,
					url: Set(l.href().to_string()),
					object: Set(object.internal),
					document_type: Set(apb::DocumentType::Page),
					name: Set(l.link_name().map(|x| x.to_string())),
					media_type: Set(l.link_media_type().unwrap_or("link").to_string()),
					created: Set(chrono::Utc::now()),
				},
				Node::Object(o) => model::attachment::ActiveModel {
					internal: sea_orm::ActiveValue::NotSet,
					url: Set(o.url().id().unwrap_or_else(|| o.id().map(|x| x.to_string()).unwrap_or_default())),
					object: Set(object.internal),
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
				internal: sea_orm::ActiveValue::NotSet,
				url: Set(img.url().id().unwrap_or_else(|| img.id().map(|x| x.to_string()).unwrap_or_default())),
				object: Set(object.internal),
				document_type: Set(img.as_document().map_or(apb::DocumentType::Document, |x| x.document_type().unwrap_or(apb::DocumentType::Page))),
				name: Set(img.name().map(|x| x.to_string())),
				media_type: Set(img.media_type().unwrap_or(media_type.as_deref().unwrap_or("link")).to_string()),
				created: Set(img.published().unwrap_or_else(chrono::Utc::now)),
			};
			model::attachment::Entity::insert(attachment_model)
				.exec(self.db())
				.await?;
		}

		Ok(object)
	}
}
