use sea_orm::EntityTrait;
use reqwest::Method;

use apb::{LD, Node, ActivityMut};
use upub::{Context, model, traits::Fetcher};

pub async fn process(ctx: Context, job: &model::job::Model) -> crate::JobResult<()> {
	tracing::info!("delivering {} to {:?}", job.activity, job.target);

	let payload = match model::activity::Entity::find_by_ap_id(&job.activity)
		.find_also_related(model::object::Entity)
		.one(ctx.db())
		.await?
	{
		Some((activity, None)) => activity.ap().ld_context(),
		Some((activity, Some(object))) => {
			let always_embed = matches!(
				activity.activity_type,
				apb::ActivityType::Create
				| apb::ActivityType::Undo
				| apb::ActivityType::Update
				| apb::ActivityType::Accept(_)
				| apb::ActivityType::Reject(_)
			);
			if always_embed {
				activity.ap().set_object(Node::object(object.ap())).ld_context()
			} else {
				activity.ap().ld_context()
			}
		},
		None => {
			tracing::info!("skipping dispatch for deleted object {}", job.activity);
			return Ok(());
		},
	};

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
		Some(&serde_json::to_string(&payload).unwrap()),
		&job.actor, &key, ctx.domain()
	).await {
		tracing::warn!("failed delivery of {} to {:?} : {e}", job.activity, job.target);
		model::job::Entity::insert(job.clone().repeat())
			.exec(ctx.db())
			.await?;
	}

	Ok(())
}
