use sea_orm::EntityTrait;
use reqwest::Method;

use apb::{LD, ActivityMut};
use upub::{Context, model, traits::Fetcher};

pub async fn process(ctx: Context, job: &model::job::Model) -> crate::JobResult<()> {
	tracing::info!("delivering {} to {:?}", job.activity, job.target);

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
					.map(|x| x.ap()),
			apb::ActivityType::Accept(_) | apb::ActivityType::Reject(_) | apb::ActivityType::Undo =>
				model::activity::Entity::find_by_ap_id(oid)
					.one(ctx.db())
					.await?
					.map(|x| x.ap()),
			apb::ActivityType::Update => {
				if let Some(o) = model::object::Entity::find_by_ap_id(oid).one(ctx.db()).await? {
					Some(o.ap())
				} else if let Some(a) = model::actor::Entity::find_by_ap_id(oid).one(ctx.db()).await? {
					Some(a.ap())
				} else {
					None
				}
			},
			_ => None,
		}
	} else { None };
	
	let mut payload = activity.ap();
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

	if let Err(e) = Context::request(
		Method::POST, job.target.as_deref().unwrap_or(""),
		Some(&serde_json::to_string(&payload.ld_context()).unwrap()),
		&job.actor, &key, ctx.domain()
	).await {
		tracing::warn!("failed delivery of {} to {:?} : {e}", job.activity, job.target);
		model::job::Entity::insert(job.clone().repeat(Some(e.to_string())))
			.exec(ctx.db())
			.await?;
	}

	Ok(())
}
