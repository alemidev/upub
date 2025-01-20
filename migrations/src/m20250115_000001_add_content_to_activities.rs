use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::Activities;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(Activities::Table)
					.add_column(ColumnDef::new(Activities::Content).string().null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(Activities::Table)
					.drop_column(Activities::Content)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

