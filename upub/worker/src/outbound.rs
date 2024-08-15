use apb::{field::OptionalString, target::Addressed, Activity, ActivityMut, Actor, Base, BaseMut, Object, ObjectMut};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, SelectColumns, TransactionTrait};
use upub::{model::{self, actor::Field}, traits::{Addresser, Processor}, Context};


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
		let activity = upub::model::activity::Entity::ap_to_internal(activity.object().id()?, &tx)
			.await?
			.ok_or_else(|| DbErr::RecordNotFound(activity.object().id().unwrap_or_default().to_string()))?;
		let notif_model = upub::model::notification::ActiveModel {
			internal: sea_orm::ActiveValue::NotSet,
			activity: sea_orm::ActiveValue::Unchanged(activity),
			actor: sea_orm::ActiveValue::Unchanged(actor),
			seen: sea_orm::ActiveValue::Set(true),
			published: sea_orm::ActiveValue::NotSet,
		};
		upub::model::notification::Entity::update(notif_model).exec(&tx).await?;
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
		.set_id(Some(&job.activity))
		.set_actor(apb::Node::link(job.actor.clone()))
		.set_published(Some(now));

	if matches!(t, apb::ObjectType::Activity(apb::ActivityType::Undo)) {
		let mut undone = activity.object().extract().ok_or(crate::JobError::MissingPayload)?;
		if undone.id().is_err() {
			let undone_target = undone.object().id().str().ok_or(crate::JobError::MissingPayload)?;
			let undone_type = undone.activity_type().map_err(|_| crate::JobError::MissingPayload)?;
			let undone_model = model::activity::Entity::find()
				.filter(model::activity::Column::Object.eq(&undone_target))
				.filter(model::activity::Column::Actor.eq(&job.actor))
				.filter(model::activity::Column::ActivityType.eq(undone_type))
				.order_by_desc(model::activity::Column::Published)
				.one(&tx)
				.await?
				.ok_or_else(|| sea_orm::DbErr::RecordNotFound(format!("actor={},type={},object={}",job.actor, undone_type, undone_target)))?;
			undone = undone
				.set_id(Some(&undone_model.id))
				.set_actor(apb::Node::link(job.actor.clone()));
		}
		activity = activity.set_object(apb::Node::object(undone));
	}

	if matches!(t, apb::ObjectType::Activity(apb::ActivityType::Update)) {
		let mut updated = activity.object().extract().ok_or(crate::JobError::MissingPayload)?;
		if updated.actor_type().is_ok() {
			let mut prev = model::actor::Entity::find_by_ap_id(updated.id()?)
				.one(&tx)
				.await?
				.ok_or_else(|| crate::JobError::MissingPayload)?;
			if prev.id != job.actor {
				return Err(crate::JobError::Forbidden);
			}
			if let Some(name) = updated.name().str() {
				prev.name = Some(name);
			}
			if let Some(summary) = updated.summary().str() {
				prev.summary = Some(summary);
			}
			if let Some(icon) = updated.icon().get().and_then(|x| x.url().id().str()) {
				prev.icon = Some(icon);
			}
			if let Some(image) = updated.image().get().and_then(|x| x.url().id().str()) {
				prev.image = Some(image);
			}
			if !updated.attachment().is_empty() {
				prev.fields = updated.attachment()
					.flat()
					.into_iter()
					.filter_map(|x| x.extract())
					.map(Field::from)
					.collect::<Vec<Field>>()
					.into();
			}
			updated = prev.ap();
		}
		activity = activity.set_object(apb::Node::object(updated));
	}

	if matches!(t, apb::ObjectType::Activity(apb::ActivityType::Create)) {
		let raw_oid = Context::new_id();
		let oid = ctx.oid(&raw_oid);
		// object must be embedded, wont dereference here
		let object = activity.object().extract().ok_or(apb::FieldErr("object"))?;
		// TODO regex hell here i come...
		let re = regex::Regex::new(r"@(.+)@([^ ]+)").expect("failed compiling regex pattern");
		let mut content = object.content().map(|x| x.to_string()).ok();
		if let Some(c) = content {
			let mut tmp = mdhtml::safe_markdown(&c);
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

		activity = activity
			.set_object(apb::Node::object(
					object
						.set_id(Some(&oid))
						.set_content(content.as_deref())
						.set_attributed_to(apb::Node::link(job.actor.clone()))
						.set_published(Some(now))
						.set_updated(Some(now))
						.set_url(apb::Node::maybe_link(ctx.cfg().frontend_url(&format!("/objects/{raw_oid}")))),
			));
	}

	// TODO very important that we limit Update activities!!! otherwise with .process() local users
	// can change their document completely

	let targets = activity.addressed();
	ctx.process(activity, &tx).await?;
	ctx.deliver(targets, &job.activity, &job.actor, &tx).await?;

	tx.commit().await?;

	Ok(())
}
