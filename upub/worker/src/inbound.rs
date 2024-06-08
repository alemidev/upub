use sea_orm::TransactionTrait;
use upub::traits::Processor;


pub async fn process(ctx: upub::Context, job: &upub::model::job::Model) -> crate::JobResult<()> {
	let Some(ref activity) = job.payload else {
		tracing::error!("abandoning inbound job without payload: {job:#?}");
		return Ok(());
	};

	let tx = ctx.db().begin().await?;
	// TODO can we get rid of this clone?
	ctx.process(activity.clone(), &tx).await?;
	tx.commit().await?;

	Ok(())
}
