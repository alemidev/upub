use sea_orm_migration::prelude::*;

use crate::m20240605_000001_add_jobs_table::Jobs;

#[derive(DeriveIden)]
pub enum JobCompletions {
	Table,
	Internal,
	Job,
	Completed,
	Duration,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.create_table(
				Table::create()
					.table(JobCompletions::Table)
					.comment("track job completions and duration time")
					.col(
						ColumnDef::new(JobCompletions::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(JobCompletions::Job).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-completions-jobs")
							.from(JobCompletions::Table, JobCompletions::Job)
							.to(Jobs::Table, Jobs::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::NoAction)
					)
					.col(ColumnDef::new(JobCompletions::Completed).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(JobCompletions::Duration).big_integer().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-job-completions-completed")
					.table(JobCompletions::Table)
					.col((JobCompletions::Completed, IndexOrder::Desc))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(JobCompletions::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-job-completions-completed").table(JobCompletions::Table).to_owned())
			.await?;

		Ok(())
	}
}
