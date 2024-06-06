use sea_orm_migration::prelude::*;

use crate::m20240605_000001_add_jobs_table::Jobs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(
				Index::drop()
					.name("index-jobs-activity")
					.table(Jobs::Table)
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-jobs-activity")
					.table(Jobs::Table)
					.col(Jobs::Activity)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
