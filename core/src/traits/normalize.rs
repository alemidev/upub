use apb::{Endpoints, Node, Object, PublicKey, Shortcuts, Question};
use sea_orm::{sea_query::Expr, ActiveModelTrait, ActiveValue::{Unchanged, NotSet, Set}, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter};

use crate::ext::{AnyQuery, TakeAsRef};

use super::{Cloaker, Fetcher};

#[derive(Debug, thiserror::Error)]
pub enum NormalizerError {
	#[error("normalized document misses required field: {0:?}")]
	Malformed(#[from] apb::FieldErr),

	#[error("wrong object type: expected {0}, got {1}")]
	WrongType(apb::BaseType, apb::BaseType),

	#[error("database error while normalizing object: {0:?}")]
	DbErr(#[from] sea_orm::DbErr),
}

#[allow(async_fn_in_trait)]
pub trait Normalizer {
	async fn insert_object(&self, obj: impl apb::Object, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, NormalizerError>;
	async fn insert_activity(&self, act: impl apb::Activity, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, NormalizerError>;
}

impl Normalizer for crate::Context {

	async fn insert_object(&self, object: impl apb::Object, tx: &impl ConnectionTrait) -> Result<crate::model::object::Model, NormalizerError> {
		let mut object_model = AP::object(&object)?;

		if let Some(content) = object_model.content {
			object_model.content = Some(self.sanitize(content));
		}

		if let Some(image) = object_model.image {
			object_model.image = Some(self.cloaked(&image));
		}

		// fix context for remote posts
		// > if any link is broken or we get rate limited, the whole insertion fails which is
		// > kind of dumb. there should be a job system so this can be done in waves. or maybe there's
		// > some whole other way to do this?? im thinking but misskey aaaa!! TODO
		if let Ok(reply) = object.in_reply_to().id() {
			if let Some(o) = crate::model::object::Entity::find_by_ap_id(&reply).one(tx).await? {
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

		let attachments = object.attachment().flat();
		let attachments_len = attachments.len();
		for attachment in attachments {
			let attachment_model = match attachment {
				Node::Empty => continue,

				Node::Array(_) => {
					tracing::warn!("ignoring array-in-array while processing attachments");
					continue
				},

				Node::Object(o) => {
					let model = AP::attachment_q(o.as_document()?, object_model.internal, None)?;
					match process_and_normalize_image(self, model, object_model.image.as_deref(), attachments_len) {
						ModelOrUrl::Neither => continue,
						ModelOrUrl::Url(u) => {
							object_model.image = Some(u);
							continue;
						},
						ModelOrUrl::Model(m) => m,
					}
				},

				Node::Link(l) => {
					let url = l.href().unwrap_or_default();
					let media_type = l.media_type().unwrap_or("text/html".to_string());
					let model = crate::model::attachment::ActiveModel {
						internal: sea_orm::ActiveValue::NotSet,
						url: Set(url),
						object: Set(object_model.internal),
						document_type: Set(apb::DocumentType::Document),
						media_type: Set(media_type),
						name: Set(l.name().ok()),
					};

					match process_and_normalize_image(self, model, object_model.image.as_deref(), attachments_len) {
						ModelOrUrl::Neither => continue,
						ModelOrUrl::Url(u) => {
							object_model.image = Some(u);
							continue;
						},
						ModelOrUrl::Model(m) => m,
					}
				},
			};

			crate::model::attachment::Entity::insert(attachment_model)
				.exec(tx)
				.await?;
		}

		for tag in object.tag().flat() {
			use apb::Link;
			if let Ok(doc) = tag.into_inner() {
				if let Ok(l) = doc.as_link() {
					match l.link_type() {
						Ok(apb::LinkType::Mention) => {
							if let Ok(href) = l.href() {
								// TODO here we do a silent fetch, in theory normalizer trait should not use fetcher
								//      trait because fetcher uses normalizer (and it becomes cyclic), however here
								//      we should try to resolve remote users mentioned, otherwise most mentions will
								//      be lost. also we shouldn't fail inserting the whole post if the mention fails
								//      resolving.
								if let Ok(user) = self.fetch_user(&href, tx).await {
									let model = crate::model::mention::ActiveModel {
										internal: NotSet,
										object: Set(object_model.internal),
										actor: Set(user.internal),
									};
									crate::model::mention::Entity::insert(model)
										.exec(tx)
										.await?;
								}
							}
						},
						Ok(apb::LinkType::Hashtag) => {
							let hashtag = l.name()
								.unwrap_or_else(|_| l.href().unwrap_or_default().split('/').next_back().unwrap_or_default().to_string()) // TODO maybe just fail?
								.replace('#', "");
							// TODO lemmy added a "fix" to make its communities kind of work with mastodon:
							//      basically they include the community name as hashtag. ughhhh, since we handle
							//      hashtags and audience it means our hashtags gets clogged with posts from lemmy
							//      communities. it kind of make sense to include them since they fit the hashtag
							//      theme, but nonetheless it's annoying and i'd rather not have the two things
							//      mixed. maybe it's just me and this should go instead? maybe this has other
							//      issues and it's just not worth fixing this tiny lemmy kink? idkk
							if let Some(ref audience) = object_model.audience {
								if audience.ends_with(&hashtag) {
									continue;
								}
							}
							let model = crate::model::hashtag::ActiveModel {
								internal: NotSet,
								object: Set(object_model.internal),
								name: Set(hashtag),
							};
							crate::model::hashtag::Entity::insert(model)
								.exec(tx)
								.await?;
						},
						Ok(apb::LinkType::Emoji) => {
							let name = l.name().unwrap_or_default().replace(':', "");
							let domain = crate::Context::server(&object_model.id);
							let uri = doc.icon().into_inner().and_then(|x| x.url().id()).unwrap_or_default();
							if !name.is_empty()
								&& !domain.is_empty()
								&& !uri.is_empty()
								&& !crate::model::emoji::Entity::find()
									.filter(crate::model::emoji::Column::Name.eq(&name))
									.filter(crate::model::emoji::Column::Domain.eq(&domain))
									.any(tx)
									.await?
								// TODO every time we resolve an user we make multiple queries
							{
								crate::model::emoji::ActiveModel {
									internal: NotSet,
									domain: Set(domain),
									name: Set(name),
									uri: Set(uri),
								}
									.insert(tx)
									.await?;
							}
						},
						_ => {},
					}
				}
			}
		}

		if let Ok(question) = object.as_question() {
			for option in question.any_of().flat() {
				if let Ok(doc) = option.into_inner() {
					crate::model::question_option::ActiveModel {
						internal: sea_orm::ActiveValue::NotSet,
						object: sea_orm::ActiveValue::Set(object_model.internal),
						name: sea_orm::ActiveValue::Set(doc.name().unwrap_or_default()),
						votes: sea_orm::ActiveValue::Set(doc.replies_count().unwrap_or_default()),
					}
						.insert(tx)
						.await?;
				}
			}
		}

		if let Ok(question) = object.as_question() {
			for option in question.one_of().flat() {
				if let Ok(doc) = option.into_inner() {
					crate::model::question_option::ActiveModel {
						internal: sea_orm::ActiveValue::NotSet,
						object: sea_orm::ActiveValue::Set(object_model.internal),
						name: sea_orm::ActiveValue::Set(doc.name().unwrap_or_default()),
						votes: sea_orm::ActiveValue::Set(doc.replies_count().unwrap_or_default()),
					}
						.insert(tx)
						.await?;
				}
			}
		}

		Ok(object_model)
	}

	async fn insert_activity(&self, activity: impl apb::Activity, tx: &impl ConnectionTrait) -> Result<crate::model::activity::Model, NormalizerError> {
		let mut activity_model = AP::activity(&activity)?;

		// TODO activity addressing normalization!
		//      since ActivityPub is a mess most software doesnt' really respect addressing, or care
		//      about inserting it correctly.
		//
		//   *  we can assume that Follow and Accept activities should *at least* be
		//      addressed to their target, since how would anyone be able to accept it otherwise???

		match activity_model.activity_type {
			apb::ActivityType::Follow
			| apb::ActivityType::Accept(apb::AcceptType::Accept)
			=> {
				if let Some(ref target) = activity_model.object {
					if !activity_model.to.0.contains(target) {
						activity_model.to.0.push(target.clone());
					}
				}
			},
			_ => {},
		}

		for tag in activity.tag().flat() {
			use apb::Link;
			if let Ok(doc) = tag.into_inner() {
				if let Ok(l) = doc.as_link() {
					if matches!(l.link_type(), Ok(apb::LinkType::Emoji)) {
						let name = l.name().unwrap_or_default().replace(':', "");
						let domain = crate::Context::server(&activity_model.id);
						let uri = doc.icon().into_inner().and_then(|x| x.url().id()).unwrap_or_default();
						if !name.is_empty()
							&& !domain.is_empty()
							&& !uri.is_empty()
							&& !crate::model::emoji::Entity::find()
								.filter(crate::model::emoji::Column::Name.eq(&name))
								.filter(crate::model::emoji::Column::Domain.eq(&domain))
								.any(tx)
								.await?
							// TODO every time we resolve an user we make multiple queries
						{
							crate::model::emoji::ActiveModel {
								internal: NotSet,
								domain: Set(domain),
								name: Set(name),
								uri: Set(uri),
							}
								.insert(tx)
								.await?;
						}
					}
				}
			}
		}

		let mut active_model = activity_model.clone().into_active_model();
		active_model.internal = NotSet;
		crate::model::activity::Entity::insert(active_model)
			.exec(tx)
			.await?;

		let internal = crate::model::activity::Entity::ap_to_internal(&activity_model.id, tx)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(activity_model.id.clone()))?;
		activity_model.internal = internal;

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
			object: activity.object().id().ok(),
			target: activity.target().id().ok(),
			content: activity.content().ok(),
			published: activity.published().unwrap_or(chrono::Utc::now()),
			to: activity.to().all_ids().into(),
			bto: activity.bto().all_ids().into(),
			cc: activity.cc().all_ids().into(),
			bcc: activity.bcc().all_ids().into(),
		})
	}

	pub fn activity_q(activity: &impl apb::Activity, internal: Option<i64>) -> Result<crate::model::activity::ActiveModel, NormalizerError> {
		let mut m = AP::activity(activity)?.into_active_model();
		m = m.reset_all();
		match internal {
			Some(x) => m.internal = Unchanged(x),
			None => m.internal = NotSet,
		}
		Ok(m)
	}




	pub fn attachment(document: &impl apb::Document, parent: i64) -> Result<crate::model::attachment::Model, NormalizerError> {
		let base_type = document.base_type()?;
		if !matches!(base_type, apb::BaseType::Object(apb::ObjectType::Document(_))) {
			return Err(NormalizerError::WrongType(apb::BaseType::Object(apb::ObjectType::Document(apb::DocumentType::Document)), base_type));
		}

		let media_type = document.media_type().unwrap_or("text/html".to_string());
		let (mime_kind, _mime_type) = media_type.split_once('/').unwrap_or_default();
		let document_type = document.document_type().unwrap_or(match mime_kind {
			"image" => apb::DocumentType::Image,
			"video" => apb::DocumentType::Video,
			"audio" => apb::DocumentType::Audio,
			"text"  => apb::DocumentType::Page,
			_ => apb::DocumentType::Document,
		});

		Ok(crate::model::attachment::Model {
			internal: 0,
			url: document.url().id().unwrap_or_default(),
			object: parent,
			name: document.name().ok(),
			media_type, document_type,
		})
	}

	pub fn attachment_q(document: &impl apb::Document, parent: i64, internal: Option<i64>) -> Result<crate::model::attachment::ActiveModel, NormalizerError> {
		let mut m = AP::attachment(document, parent)?.into_active_model();
		m = m.reset_all();
		match internal {
			Some(x) => m.internal = Unchanged(x),
			None => m.internal = NotSet,
		}
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
				| apb::ObjectType::Activity(apb::ActivityType::IntransitiveActivity(apb::IntransitiveActivityType::Question))
				| apb::ObjectType::Document(apb::DocumentType::Page) // why Document lemmy??????
			)
		) {
			return Err(NormalizerError::WrongType(apb::BaseType::Object(apb::ObjectType::Object), t));
		}

		let is_multiple_choice_poll = match object.as_question() {
			Ok(question) => Some(!question.any_of().is_empty()),
			Err(_) => None,
		};

		Ok(crate::model::object::Model {
			internal: 0,
			id: object.id()?.to_string(),
			object_type: object.object_type()?,
			attributed_to: object.attributed_to().id().ok(),
			name: object.name().ok(),
			summary: object.summary().ok(),
			content: object.content().ok(),
			image: object.image_url().ok(),
			context: object.context().id().ok(),
			in_reply_to: object.in_reply_to().id().ok(),
			quote: object.quote_url().id().ok(),
			published: object.published().unwrap_or_else(|_| chrono::Utc::now()),
			updated: object.updated().unwrap_or_else(|_| chrono::Utc::now()),
			url: object.url().id().ok(),
			replies: 0, // dont count replies we don't have, since when we pull them they just get added
			likes: object.likes_count().unwrap_or_default(),
			announces: object.shares_count().unwrap_or_default(),
			audience: object.audience().id().ok(),
			to: object.to().all_ids().into(),
			bto: object.bto().all_ids().into(),
			cc: object.cc().all_ids().into(),
			bcc: object.bcc().all_ids().into(),
			end_time: object.end_time().ok(),
			sensitive: object.sensitive().unwrap_or(false),
			is_multiple_choice_poll,
		})
	}

	pub fn object_q(object: &impl apb::Object, internal: Option<i64>) -> Result<crate::model::object::ActiveModel, NormalizerError> {
		let mut m = AP::object(object)?.into_active_model();
		m = m.reset_all();
		match internal {
			Some(x) => m.internal = Unchanged(x),
			None => m.internal = NotSet,
		}
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
			let last = splits.next_back().unwrap_or(first);
			(first.to_string(), last.to_string())
		};
		Ok(crate::model::actor::Model {
			internal: 0,
			domain,
			id: ap_id,
			preferred_username: actor.preferred_username().unwrap_or(fallback_preferred_username).to_string(),
			actor_type: actor.actor_type()?,
			name: actor.name().ok(),
			summary: actor.summary().ok(),
			icon: actor.icon_url().ok(),
			image: actor.image_url().ok(),
			inbox: actor.inbox().id().ok(),
			outbox: actor.outbox().id().ok(),
			shared_inbox: actor.endpoints().inner().and_then(|x| x.shared_inbox()).map(|x| x.to_string()).ok(),
			followers: actor.followers().id().ok(),
			following: actor.following().id().ok(),
			also_known_as: actor.also_known_as().flat().into_iter().filter_map(|x| x.id().ok()).collect::<Vec<String>>().into(),
			moved_to: actor.moved_to().id().ok(),
			published: actor.published().unwrap_or(chrono::Utc::now()),
			updated: chrono::Utc::now(),
			following_count: actor.following_count().unwrap_or(0) as i32,
			followers_count: actor.followers_count().unwrap_or(0) as i32,
			statuses_count: actor.statuses_count().unwrap_or(0) as i32,
			public_key: actor.public_key().inner()?.public_key_pem().to_string(),
			private_key: None, // there's no way to transport privkey over AP json, must come from DB
			fields: actor.attachment()
				.flat()
				.into_iter()
				.filter_map(|x| Some(crate::model::actor::Field::from(x.into_inner().ok()?)))
				.collect::<Vec<crate::model::actor::Field>>()
				.into(),
		})
	}

	pub fn actor_q(actor: &impl apb::Actor, internal: Option<i64>) -> Result<crate::model::actor::ActiveModel, NormalizerError> {
		let mut m = AP::actor(actor)?.into_active_model();
		m = m.reset_all();
		m.private_key = NotSet;
		match internal {
			Some(x) => m.internal = Unchanged(x),
			None => m.internal = NotSet,
		}
		Ok(m)
	}
}





enum ModelOrUrl {
	Model(crate::model::attachment::ActiveModel),
	Url(String),
	Neither,
}

// this is a bit jank but its purpose is mostly to centralize attachment handling
fn process_and_normalize_image(
	ctx: &crate::Context,
	mut model: crate::model::attachment::ActiveModel,
	obj_image: Option<&str>,
	attachments_len: usize,
) -> ModelOrUrl {
	let Some(url) = model.url.take_as_ref().cloned() else { return ModelOrUrl::Neither };
	if let Some(obj_image_url) = obj_image {
		if url == obj_image_url { return ModelOrUrl::Neither };
	}
	model.url = Set(ctx.cloaked(&url));

	if ctx.cfg().compat.fix_attachment_media_type && model.document_type == Set(apb::DocumentType::Document) {
		let media_type = model.media_type.clone().take().unwrap_or_default();
		let (mime_kind, _mime_type) = media_type.split_once('/').unwrap_or_default();
		model.document_type = Set(match mime_kind {
			"image" => apb::DocumentType::Image,
			"video" => apb::DocumentType::Video,
			"audio" => apb::DocumentType::Audio,
			"text"  => apb::DocumentType::Page,
			_ => apb::DocumentType::Document,
		});
	}

	// in case we get both broken media_type and document_type, try to fix images with url
	// TODO is this still needed? above case with mediaType should solve most issues
	let mut is_image = matches!(model.document_type, Set(apb::DocumentType::Image));
	if [".jpg", ".jpeg", ".png", ".webp", ".bmp", ".gif", ".ico", ".svg"] // TODO more image types???
		.iter()
		.any(|x| url.ends_with(x))
	{
		is_image = true;
		if ctx.cfg().compat.fix_attachment_media_type {
			model.document_type = Set(apb::DocumentType::Image);
			model.media_type = Set(format!("image/{}", url.split('.').next_back().unwrap_or_default()));
		}

	}

	// TODO this check is a bit disgusting but lemmy for some incomprehensible reason sends us
	// the same image twice: once in `image` and once as `attachment`. you may say "well just
	// check if url is the same" and i absolutely do but lemmy is 10 steps forwards and it sends
	// the same image twice with two distinct links. checkmate fedi developers!!!!!
	// so basically i don't want to clutter my timeline with double images, nor fetch every image
	// that comes from lemmy (we cloak and lazy-load) just to dedupe it...
	if is_image
		&& ctx.cfg().compat.skip_single_attachment_if_image_is_set
		&& obj_image.is_some()
		&& attachments_len == 1
	{
		return ModelOrUrl::Url(url);
	}

	ModelOrUrl::Model(model)
}
