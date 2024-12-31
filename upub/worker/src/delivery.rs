use apb::{LD, ActivityMut};
use upub::{Context, model, traits::Fetcher};

#[allow(clippy::manual_map)] // TODO can Update code be improved?
pub async fn process(ctx: Context, job: &model::job::Model) -> crate::JobResult<()> {
	let Some(ref target) = job.target else {
		return Err(crate::JobError::Malformed(apb::FieldErr("target"))); // TODO not best error to use..
	};

	if upub::ext::is_blacklisted(target, &ctx.cfg().reject.delivery) {
		tracing::warn!("aborting delivery to {target} due to rejection policies: {:?}", job.payload);
		return Ok(());
	}

	tracing::info!("delivering {} to {target}", job.activity);

	let Some(activity) = model::activity::Entity::find_by_ap_id(&job.activity)
		.one(ctx.db())
		.await?
	else {
		tracing::info!("skipping dispatch for deleted object {}", job.activity);
		return Ok(());
	};

	let object = if let Some(ref oid) = activity.object {
		match activity.activity_type {
			apb::ActivityType::Create =>
				model::object::Entity::find_by_ap_id(oid)
					.one(ctx.db())
					.await?
					.map(|x| ctx.ap(x)),
			apb::ActivityType::Accept(_) | apb::ActivityType::Reject(_) | apb::ActivityType::Undo =>
				model::activity::Entity::find_by_ap_id(oid)
					.one(ctx.db())
					.await?
					.map(|x| ctx.ap(x)),
			apb::ActivityType::Update => {
				if let Some(o) = model::object::Entity::find_by_ap_id(oid).one(ctx.db()).await? {
					Some(ctx.ap(o))
				} else if let Some(a) = model::actor::Entity::find_by_ap_id(oid).one(ctx.db()).await? {
					Some(ctx.ap(a))
				} else {
					None
				}
			},
			_ => None,
		}
	} else { None };
	
	let mut payload = ctx.ap(activity);
	if let Some(object) = object {
		payload = payload.set_object(apb::Node::object(object));
	}

	let Some(actor) = model::actor::Entity::find_by_ap_id(&job.actor)
		.one(ctx.db())
		.await?
	else {
		tracing::error!("abandoning delivery from non existant actor {}: {job:#?}", job.actor);
		return Ok(());
	};

	let Some(key) = actor.private_key
	else {
		tracing::error!("abandoning delivery from actor without private key {}: {job:#?}", job.actor);
		return Ok(());
	};

	Context::request(
		reqwest::Method::POST, target,
		Some(&serde_json::to_string(&payload.ld_context()).unwrap()),
		&job.actor, &key, ctx.domain()
	).await?;

	Ok(())
}
