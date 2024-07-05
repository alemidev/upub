use apb::{field::OptionalString, target::Addressed, Activity, ActivityMut, Base, BaseMut, Object, ObjectMut};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, SelectColumns, TransactionTrait};
use upub::{model, traits::{Addresser, Processor}, Context};


pub async fn process(ctx: Context, job: &model::job::Model) -> crate::JobResult<()> {
	// TODO can we get rid of this cloned??
	let now = chrono::Utc::now();
	let mut activity = job.payload.as_ref().cloned().ok_or(crate::JobError::MissingPayload)?;
	let mut t = activity.object_type()?;
	let tx = ctx.db().begin().await?;

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
