use apb::{target::Addressed, Activity, ActivityMut, ActorMut, Base, BaseMut, DocumentMut, Object, ObjectMut, Shortcuts};
use sea_orm::{prelude::Expr, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, SelectColumns, TransactionTrait};
use upub::{model::{self, actor::Field}, traits::{process::ProcessorError, Addresser, Processor}, Context};


pub async fn process(ctx: Context, job: &model::job::Model) -> crate::JobResult<()> {
	// TODO can we get rid of this cloned??
	let now = chrono::Utc::now();
	let mut activity = job.payload.as_ref().cloned().ok_or(crate::JobError::MissingPayload)?;
	let mut t = activity.object_type()?;
	let tx = ctx.db().begin().await?;

	// TODO this is a bit of a magic case: it just marks as viewed and returns. this because marking
	//      notifications as seen is a very internal thing to do and should not be in .process()
	//      probably. still this feels a bit dirty to do, is there a better place to do it?
	if matches!(t, apb::ObjectType::Activity(apb::ActivityType::View)) {
		let actor = upub::model::actor::Entity::ap_to_internal(&job.actor, &tx)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(job.actor.clone()))?;
		let activity = upub::model::activity::Entity::ap_to_internal(&activity.object().id()?, &tx)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(activity.object().id().unwrap_or_default().to_string()))?;
		upub::model::notification::Entity::update_many()
			.filter(upub::model::notification::Column::Activity.eq(activity))
			.filter(upub::model::notification::Column::Actor.eq(actor))
			.col_expr(upub::model::notification::Column::Seen, Expr::value(true))
			.exec(&tx).await?;
		tx.commit().await?;
		return Ok(());
	}

	if matches!(t, apb::ObjectType::Note) {
		activity = apb::new()
			.set_activity_type(Some(apb::ActivityType::Create))
			.set_to(activity.to())
			.set_bto(activity.bto())
			.set_cc(activity.cc())
			.set_bcc(activity.bcc())
			.set_object(apb::Node::object(activity));
		t = apb::ObjectType::Activity(apb::ActivityType::Create);
	}

	activity = activity
		.set_id(Some(job.activity.clone()))
		.set_actor(apb::Node::link(job.actor.clone()))
		.set_published(Some(now));

	if matches!(t, apb::ObjectType::Activity(apb::ActivityType::Undo)) {
		match activity.object().id() {
			Ok(undone) => {
				let activity = upub::model::activity::Entity::find_by_ap_id(&undone)
					.one(&tx)
					.await?
					.ok_or_else(|| DbErr::RecordNotFound(undone))?;
				if activity.actor != job.actor {
					return Err(crate::JobError::Forbidden);
				}
			},
			Err(_) => {
				// frontend doesn't know the activity id, so we have to look it up
				let undone = activity.object().into_inner()?; // if even this is missing, malformed
				match undone.activity_type()? {
					apb::ActivityType::Follow => {
						let follower = undone.actor().id().unwrap_or(job.actor.clone());
						let follower_internal = upub::model::actor::Entity::ap_to_internal(&follower, &tx)
							.await?
							.ok_or(sea_orm::DbErr::RecordNotFound(follower))?;
						let following = undone.object().id()?;
						let following_internal = upub::model::actor::Entity::ap_to_internal(&following, &tx)
							.await?
							.ok_or(sea_orm::DbErr::RecordNotFound(following))?;
						let activity_id_internal = upub::model::relation::Entity::find()
							.filter(upub::model::relation::Column::Follower.eq(follower_internal))
							.filter(upub::model::relation::Column::Following.eq(following_internal))
							.select_only()
							.select_column(upub::model::relation::Column::Activity)
							.into_tuple::<i64>()
							.one(&tx)
							.await?
							.ok_or(crate::JobError::ProcessorError(ProcessorError::Incomplete(apb::ActivityType::Follow, format!("[{follower_internal}] follows [{following_internal}]"))))?;
						let activity_id = upub::model::activity::Entity::find_by_id(activity_id_internal)
							.select_only()
							.select_column(upub::model::activity::Column::Id)
							.into_tuple::<String>()
							.one(&tx)
							.await?
							.ok_or(crate::JobError::ProcessorError(ProcessorError::Incomplete(apb::ActivityType::Follow, format!("Activity[{activity_id_internal}]"))))?;

						activity = activity.set_object(apb::Node::link(activity_id));
					},
					apb::ActivityType::Like => {
						let un_liked_object = undone.object().id()?;
						let activity_id = upub::model::activity::Entity::find()
							.select_only()
							.select_column(upub::model::activity::Column::Id)
							.filter(upub::model::activity::Column::Actor.eq(&job.actor))
							.filter(upub::model::activity::Column::Object.eq(&un_liked_object))
							.filter(upub::model::activity::Column::ActivityType.eq(apb::ActivityType::Like))
							.order_by_desc(upub::model::activity::Column::Published)
							.into_tuple::<String>()
							.one(&tx)
							.await?
							.ok_or(sea_orm::DbErr::RecordNotFound(format!("Like({un_liked_object})")))?;

						activity = activity.set_object(apb::Node::link(activity_id));
					},
					apb::ActivityType::Announce => {
						let un_announced_object = undone.object().id()?;
						let activity_id = upub::model::activity::Entity::find()
							.select_only()
							.select_column(upub::model::activity::Column::Id)
							.filter(upub::model::activity::Column::Actor.eq(&job.actor))
							.filter(upub::model::activity::Column::Object.eq(&un_announced_object))
							.filter(upub::model::activity::Column::ActivityType.eq(apb::ActivityType::Announce))
							.order_by_desc(upub::model::activity::Column::Published)
							.into_tuple::<String>()
							.one(&tx)
							.await?
							.ok_or(sea_orm::DbErr::RecordNotFound(format!("Announce({un_announced_object})")))?;

						activity = activity.set_object(apb::Node::link(activity_id));
					},
					t => return Err(crate::JobError::ProcessorError(
						ProcessorError::Unprocessable(activity.activity_type()?, format!("can't normalize Undo({t})"))
					)),
				}
			},
		}
	}

	macro_rules! update {
		($prev:ident, $field:ident, $getter:expr) => {
			if let Ok($field) = $getter {
				$prev.$field = Some($field.to_string());
			}
		};
	}

	if matches!(t, apb::ObjectType::Activity(apb::ActivityType::Update)) {
		let mut updated = activity.object().into_inner()?;
		match updated.object_type()? {
			apb::ObjectType::Actor(_) => {
				let mut prev = model::actor::Entity::find_by_ap_id(&updated.id()?)
					.one(&tx)
					.await?
					.ok_or(crate::JobError::MissingPayload)?;

				if prev.id != job.actor {
					return Err(crate::JobError::Forbidden);
				}
				
				update!(prev, name, updated.name());
				update!(prev, summary, updated.summary());
				update!(prev, icon, updated.icon_url());
				update!(prev, image, updated.image_url());

				if !updated.attachment().is_empty() {
					prev.fields = updated.attachment()
						.flat()
						.into_iter()
						.filter_map(|x| x.into_inner().ok())
						.map(Field::from)
						.collect::<Vec<Field>>()
						.into();
				}

				// TODO ughhh since following/followers count are sensitive we hide them inside the .ap()
				//      method, but this means here we end up with these set to 0 here! every time any
				//      local user updates their profile, they also reset their followers/following
				//      counters... not great! so we have to overwrite _back_ the counts here so that they
				//      don't get set to 0 later. this is a ridicolous design and a comically bad botch....
				let (following_count, followers_count) = (prev.following_count as u64, prev.followers_count as u64);
				updated = ctx.ap(prev)
					.set_following_count(Some(following_count))
					.set_followers_count(Some(followers_count));
			},
			apb::ObjectType::Note => {
				let mut prev = model::object::Entity::find_by_ap_id(&updated.id()?)
					.one(&tx)
					.await?
					.ok_or(crate::JobError::MissingPayload)?;

				if prev.attributed_to.as_ref() != Some(&job.actor) {
					return Err(crate::JobError::Forbidden);
				}

				update!(prev, name, updated.name());
				update!(prev, summary, updated.summary());
				update!(prev, content, updated.content());
				update!(prev, image, updated.image_url());

				if let Ok(sensitive) = updated.sensitive() {
					prev.sensitive = sensitive;
				}

				updated = ctx.ap(prev);
			},
			apb::ObjectType::Collection(apb::CollectionType::Collection) => {
				let mut prev = model::list::Entity::find_by_ap_id(&updated.id()?)
					.one(&tx)
					.await?
					.ok_or(crate::JobError::MissingPayload)?;

				if prev.attributed_to != job.actor {
					return Err(crate::JobError::Forbidden);
				}

				update!(prev, name, updated.name());
				update!(prev, summary, updated.summary());
				
				updated = ctx.ap(prev);
			},
			t => return Err(crate::JobError::ProcessorError(ProcessorError::Unprocessable(activity.activity_type()?, format!("{t}")))),
		}
		activity = activity.set_object(apb::Node::object(updated));
	}

	if matches!(t, apb::ObjectType::Activity(apb::ActivityType::Create)) {
		let raw_oid = Context::new_id();
		let oid = ctx.oid(&raw_oid);
		// object must be embedded, wont dereference here
		let object = activity.object().into_inner()?;
		// TODO regex hell here i come...
		let re = regex::Regex::new(r"@(.+)@([^ ]+)").expect("failed compiling regex pattern");
		let mut content = object.content().map(|x| x.to_string()).ok();
		if let Some(c) = content {
			let mut tmp = mdhtml::safe_markdown(c);
			for (full, [user, domain]) in re.captures_iter(&tmp.clone()).map(|x| x.extract()) {
				if let Ok(Some(uid)) = model::actor::Entity::find()
					.filter(model::actor::Column::PreferredUsername.eq(user))
					.filter(model::actor::Column::Domain.eq(domain))
					.select_only()
					.select_column(model::actor::Column::Id)
					.into_tuple::<String>()
					.one(&tx)
					.await
				{
					tmp = tmp.replacen(full, &format!("<a href=\"{uid}\" class=\"u-url mention\">@{user}</a>"), 1);
				}
			}
			content = Some(tmp);
		}

		let mut normalized_attachments = Vec::new();

		for attachment_node in object.attachment().flat() {
			match attachment_node.into_inner() {
				Err(e) => tracing::warn!("discarding outbound attachment: {e}"),
				Ok(mut attachment) => {
					if attachment.media_type().is_err() {
						let url = attachment.url().id().unwrap_or_default();
						if let Some(ext) = url.split('.').next_back() {
							if let Some((ty, mime)) = ext_to_mime_and_type(ext) {
								attachment = attachment
									.set_document_type(Some(ty))
									.set_media_type(Some(mime.to_string()));
							}
						}
					}
					normalized_attachments.push(attachment);
				},
			}
		}

		activity = activity
			.set_object(apb::Node::object(
					object
						.set_id(Some(oid))
						.set_content(content)
						.set_attributed_to(apb::Node::link(job.actor.clone()))
						.set_attachment(apb::Node::array(normalized_attachments))
						.set_published(Some(now))
						.set_updated(Some(now))
						.set_url(apb::Node::maybe_link(ctx.cfg().frontend_url(&format!("/objects/{raw_oid}")))),
			));
	}

	// TODO very important that we limit Update activities!!! otherwise with .process() local users
	// can change their document completely

	let mut targets = activity.addressed();
	let is_broadcast = activity.to().flat().into_iter().any(|x| apb::target::is_public(&x.id().unwrap_or_default()));
	ctx.process(activity, &tx).await?;

	if is_broadcast {
		for relay in upub::Query::related(None, Some(ctx.actor().internal), false)
			.select_only()
			.select_column(upub::model::actor::Column::Id)
			.into_tuple::<String>()
			.all(&tx)
			.await?
		{
			targets.push(relay);
		}
	}

	targets
		.retain(|target| {
			if upub::ext::is_blacklisted(target, &ctx.cfg().reject.delivery) {
				tracing::warn!("rejecting delivery of {} to {target}", job.activity);
				false
			} else {
				true
			}
		});

	ctx.deliver(targets, &job.activity, &job.actor, &tx).await?;

	tx.commit().await?;

	ctx.wake_workers(); // dispatch immediately

	Ok(())
}

fn ext_to_mime_and_type(ext: &str) -> Option<(apb::DocumentType, &'static str)> {
	match ext {
		"jpg" | "jpeg" => Some((apb::DocumentType::Image, "image/jpeg")),
		"png" => Some((apb::DocumentType::Image, "image/png")),
		"webp" => Some((apb::DocumentType::Image, "image/webp")),
		"gif" => Some((apb::DocumentType::Image, "image/gif")),
		"svg" => Some((apb::DocumentType::Image, "image/svg+xml")),
		"ico" => Some((apb::DocumentType::Image, "image/vnd.microsoft.icon")),
		"bmp" => Some((apb::DocumentType::Image, "image/bmp")),
		"aac" => Some((apb::DocumentType::Audio, "audio/aac")),
		"mid" | "midi" => Some((apb::DocumentType::Audio, "audio/midi")),
		"mp3" => Some((apb::DocumentType::Audio, "audio/mpeg")),
		"oga" | "opus" => Some((apb::DocumentType::Audio, "audio/ogg")),
		"wav" => Some((apb::DocumentType::Audio, "audio/wav")),
		"weba" => Some((apb::DocumentType::Audio, "audio/webm")),
		"avi" => Some((apb::DocumentType::Video, "video/x-msvideo")),
		"mp4" => Some((apb::DocumentType::Video, "video/mp4")),
		"mpeg" => Some((apb::DocumentType::Video, "video/mpeg")),
		"ogv" => Some((apb::DocumentType::Video, "video/ogg")),
		"webm" => Some((apb::DocumentType::Video, "video/webm")),
		"css" => Some((apb::DocumentType::Document, "text/css")),
		"csv" => Some((apb::DocumentType::Document, "text/csv")),
		"htm" | "html" => Some((apb::DocumentType::Page, "text/html")),
		"js" => Some((apb::DocumentType::Document, "text/javascript")),
		"md" => Some((apb::DocumentType::Document, "text/markdown")),
		"txt" => Some((apb::DocumentType::Document, "text/plain")),
		_ => None,
	}
}
