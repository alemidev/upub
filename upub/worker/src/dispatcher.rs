use sea_orm::{ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder};

use upub::{model, Context};

#[derive(Debug, thiserror::Error)]
pub enum JobError {
	#[error("database error: {0:?}")]
	Database(#[from] sea_orm::DbErr),

	#[error("invalid payload json: {0:?}")]
	Json(#[from] serde_json::Error),

	#[error("malformed payload: {0}")]
	Malformed(#[from] apb::FieldErr),

	#[error("malformed job: missing payload")]
	MissingPayload,

	#[error("no available implementation to process job")]
	Unprocessable,

	#[error("error processing activity: {0:?}")]
	ProcessorError(#[from] upub::traits::process::ProcessorError),
}

pub type JobResult<T> = Result<T, JobError>;

#[async_trait::async_trait]
pub trait JobDispatcher : Sized {
	async fn poll(&self, filter: Option<model::job::JobType>) -> JobResult<Option<model::job::Model>>;
	async fn lock(&self, job_internal: i64) -> JobResult<bool>;
	async fn run(self, concurrency: usize, poll_interval: u64, job_filter: Option<model::job::JobType>);
}

#[async_trait::async_trait]
impl JobDispatcher for Context {
	async fn poll(&self, filter: Option<model::job::JobType>) -> JobResult<Option<model::job::Model>> {
		let mut s = model::job::Entity::find()
			.filter(model::job::Column::NotBefore.lte(chrono::Utc::now()));

		if let Some(t) = filter {
			s = s.filter(model::job::Column::JobType.eq(t));
		}
		
		Ok(
			s
				.order_by(model::job::Column::NotBefore, Order::Asc)
				.one(self.db())
				.await?
		)
	}

	async fn lock(&self, job_internal: i64) -> JobResult<bool> {
		let res = model::job::Entity::delete(
			model::job::ActiveModel {
				internal: sea_orm::ActiveValue::Set(job_internal),
				..Default::default()
			}
		)
			.exec(self.db())
			.await?;

		if res.rows_affected < 1 {
			return Ok(false);
		}

		Ok(true)
	}

	async fn run(self, concurrency: usize, poll_interval: u64, job_filter: Option<model::job::JobType>) {
		macro_rules! restart {
			(now) => { continue };
			() => {
				{
					tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
					continue;
				}
			}
		}

		let mut pool = tokio::task::JoinSet::new();
	
		loop {
			let job = match self.poll(job_filter).await {
				Ok(Some(j)) => j,
				Ok(None) => restart!(),
				Err(e) => {
					tracing::error!("error polling for jobs: {e}");
					restart!()
				},
			};
	
			match self.lock(job.internal).await {
				Ok(true) => {},
				Ok(false) => restart!(now),
				Err(e) => {
					tracing::error!("error locking job: {e}");
					restart!()
				},
			}
	
			if chrono::Utc::now() > job.published + chrono::Duration::days(self.cfg().security.job_expiration_days as i64) {
				tracing::info!("dropping expired job {job:?}");
				restart!(now);
			}

			if let Ok(Some(_)) = model::activity::Entity::find_by_ap_id(&job.activity)
				.one(self.db())
				.await
			{
				tracing::info!("dropping already processed job '{}'", job.activity);
				restart!(now);
			}

			let _ctx = self.clone();
			pool.spawn(async move {
				let res = match job.job_type {
					model::job::JobType::Inbound => crate::inbound::process(_ctx.clone(), &job).await,
					model::job::JobType::Outbound => crate::outbound::process(_ctx.clone(), &job).await,
					model::job::JobType::Local => crate::local::process(_ctx.clone(), &job).await,
				};
	
				if let Err(e) = res {
					tracing::error!("failed processing job '{}': {e}", job.activity);
					let active = job.clone().repeat();
					let mut count = 0;
					loop {
						match model::job::Entity::insert(active.clone()).exec(_ctx.db()).await {
							Err(e) => tracing::error!("could not insert back job '{}': {e}", job.activity),
							Ok(_) => break,
						}
						count += 1;
						if count > _ctx.cfg().security.reinsertion_attempt_limit {
							tracing::error!("reached job reinsertion limit, dropping {job:#?}");
							break;
						}
						tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
					}
				}
			});

			while pool.len() >= concurrency {
				if let Some(Err(e)) =  pool.join_next().await {
					tracing::error!("failed joining processing task: {e}");
				}
			}
		}
	}
}
