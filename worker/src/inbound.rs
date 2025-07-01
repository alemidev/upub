use apb::{Activity, Base};
use sea_orm::TransactionTrait;
use upub::traits::{Fetcher, Processor};


pub async fn process(ctx: upub::Context, job: &upub::model::job::Model) -> crate::JobResult<()> {
	let Some(mut activity) = job.payload.clone() else {
		tracing::error!("abandoning inbound job without payload: {job:#?}");
		return Ok(());
	};

	let activity_actor = activity.actor().id()?;

	if job.actor != activity_actor {
		if ctx.cfg().compat.verify_relayed_activities_by_fetching {
			activity = ctx.pull(&activity.id()?).await?.activity()?;
		} else {
			// this should not happen since we 403 directly while queueing if compat option isn't set,
			//  however this job could have been queued for a while and config changed in the meantime
			tracing::error!("discarding job: actor {activity_actor} doesn't match {}", job.actor);
			return Ok(());
		}
	}

	let tx = ctx.db().begin().await?;
	// TODO can we get rid of this clone?
	ctx.process(activity, &tx).await?;
	tx.commit().await?;

	Ok(())
}
