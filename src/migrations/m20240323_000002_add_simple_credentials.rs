use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Credentials::Table)
					.col(
						ColumnDef::new(Credentials::Id)
							.string()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Credentials::Email).boolean().not_null())
					.col(ColumnDef::new(Credentials::Password).boolean().not_null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Credentials::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Credentials {
	Table,
	Id,
	Email,
	Password,
}
