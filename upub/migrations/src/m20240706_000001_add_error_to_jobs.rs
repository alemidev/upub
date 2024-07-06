use sea_orm_migration::prelude::*;

use crate::m20240605_000001_add_jobs_table::Jobs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Jobs::Table)
					.add_column(ColumnDef::new(Jobs::Error).string().null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Jobs::Table)
					.drop_column(Jobs::Error)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
