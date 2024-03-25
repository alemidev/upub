use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Application::Table)
					.col(
						ColumnDef::new(Application::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Application::PrivateKey).string().not_null())
					.col(ColumnDef::new(Application::PublicKey).string().not_null())
					.col(ColumnDef::new(Application::Created).date_time().not_null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Application::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Application {
	Table,
	Id,
	PrivateKey,
	PublicKey,
	Created,
}
