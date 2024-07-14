use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::Objects;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.add_column(ColumnDef::new(Objects::Quote).string().null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.drop_column(Objects::Quote)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
