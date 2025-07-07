use apb::{Activity, Base};
use sea_orm::TransactionTrait;
use upub::{ext::AnyQuery, traits::{Fetcher, Processor}};


pub async fn process(ctx: upub::Context, job: &upub::model::job::Model) -> crate::JobResult<()> {
	let Some(mut activity) = job.payload.clone() else {
		tracing::error!("abandoning inbound job without payload: {job:#?}");
		return Ok(());
	};

	let activity_actor = activity.actor().id()?;

	if job.actor != activity_actor {
		activity = try_verifying_relayed_activity(&ctx, activity, &job.actor).await?;
	}

	let tx = ctx.db().begin().await?;
	// TODO can we get rid of this clone?
	ctx.process(activity, &tx).await?;
	tx.commit().await?;

	Ok(())
}

async fn try_verifying_relayed_activity(ctx: &upub::Context, activity: serde_json::Value, job_actor: &str) -> crate::JobResult<serde_json::Value> {
	if ctx.cfg().compat.trust_relayed_activities_by_registered_relays {
		if let Some(internal) = upub::model::actor::Entity::ap_to_internal(job_actor, ctx.db()).await? {
			if upub::Query::related(Some(ctx.actor().internal), Some(internal), false)
				.any(ctx.db())
				.await?
			{
				return Ok(activity);
			}
		};
	}

	if ctx.cfg().compat.verify_relayed_activities_by_fetching {
		// masodon generates fake activities which we can't resolve, so when they get relayed to us we
		//  can't really reoslve them. it's even worse: these activities have broken URIs since they
		//  contain fragments, which get stripped (idk if by reqwest or mastodon backend) and so this
		//  pull gets the base actor instead of the related activity, or the base note instead of its
		//  delete. we stack up tons of "Mismatched" or "Not Found" errors. to avoid this, ignore
		//  pull and activity conversion errors and just go forward with the "Forbidden" error
		if let Ok(doc) = ctx.pull(&activity.id()?).await {
			if let Ok(activity) = doc.activity() {
				return Ok(activity);
			}
		}
	}

	Err(crate::JobError::Forbidden)
}
