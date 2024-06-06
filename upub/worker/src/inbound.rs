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

	if let Err(e) = ctx.process(activity).await {
		tracing::error!("failed processing job #{}: {e}", job.internal);
	}

	Ok(())
}
