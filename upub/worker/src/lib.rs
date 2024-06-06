pub mod dispatcher;
pub mod inbound;
pub mod outbound;
pub mod local;

pub use dispatcher::{JobError, JobResult};

pub fn spawn(
	ctx: upub::Context,
	concurrency: usize,
	poll: u64,
	filter: Option<upub::model::job::JobType>,
) -> tokio::task::JoinHandle<()> {
	use dispatcher::JobDispatcher;
	tokio::spawn(async move {
		ctx.run(concurrency, poll, filter).await
	})
}
