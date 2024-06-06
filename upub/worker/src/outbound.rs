use apb::{target::Addressed, Activity, ActivityMut, BaseMut, Object, ObjectMut};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, SelectColumns, TransactionTrait};
use upub::{model, traits::{Addresser, Processor}, Context};


pub async fn process(ctx: Context, job: &model::job::Model) -> crate::JobResult<()> {
	let payload = job.payload.as_ref().ok_or(crate::JobError::MissingPayload)?;
	let mut activity : serde_json::Value = serde_json::from_str(payload)?;
	let mut t = activity.object_type()?;
	let tx = ctx.db().begin().await?;

	if matches!(t, apb::ObjectType::Note) {
		activity = apb::new()
			.set_activity_type(Some(apb::ActivityType::Create))
			.set_object(apb::Node::object(activity));
		t = apb::ObjectType::Activity(apb::ActivityType::Create);
	}

	activity = activity
		.set_id(Some(&job.activity))
		.set_actor(apb::Node::link(job.actor.clone()))
		.set_published(Some(chrono::Utc::now()));

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
						.set_published(Some(chrono::Utc::now()))
						.set_url(apb::Node::maybe_link(ctx.cfg().frontend_url(&format!("/objects/{raw_oid}")))),
			));
	}

	// TODO we expand addressing twice, ugghhhhh
	let targets = ctx.expand_addressing(activity.addressed(), &tx).await?;

	ctx.process(activity, &tx).await?;

	ctx.deliver_to(&job.activity, &job.actor, &targets, &tx).await?;

	tx.commit().await?;

	Ok(())
}
