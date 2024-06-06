use sea_orm::TransactionTrait;
use upub::traits::Processor;


pub async fn process(ctx: upub::Context, job: &upub::model::job::Model) -> crate::JobResult<()> {
	let Some(ref payload) = job.payload else {
		tracing::error!("abandoning inbound job without payload: {job:#?}");
		return Ok(());
	};

	let Ok(activity) = serde_json::from_str::<serde_json::Value>(payload) else {
		tracing::error!("abandoning inbound job with invalid payload: {job:#?}");
		return Ok(());
	};

	let tx = ctx.db().begin().await?;
	ctx.process(activity, &tx).await?;
	tx.commit().await?;

	Ok(())
}
