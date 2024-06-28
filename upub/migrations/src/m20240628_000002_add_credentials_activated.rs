use sea_orm_migration::prelude::*;

use crate::m20240524_000003_create_users_auth_and_config::Credentials;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(Credentials::Table)
					.add_column(ColumnDef::new(Credentials::Active).boolean().not_null().default(false))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(Credentials::Table)
					.drop_column(Credentials::Active)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

