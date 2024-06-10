pub mod dispatcher;
pub mod inbound;
pub mod outbound;
pub mod delivery;

pub use dispatcher::{JobError, JobResult};

pub fn spawn(
	ctx: upub::Context,
	concurrency: usize,
	poll: u64,
	filter: Option<upub::model::job::JobType>,
	stop: impl StopToken,
) -> tokio::task::JoinHandle<()> {
	use dispatcher::JobDispatcher;
	tokio::spawn(async move {
		tracing::info!("starting worker task");
		ctx.run(concurrency, poll, filter, stop).await
	})
}

pub trait StopToken: Sync + Send + 'static {
	fn stop(&self) -> bool;
}
