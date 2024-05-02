use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.add_column(ColumnDef::new(Objects::Updated).date_time().null())
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
					.drop_column(Objects::Updated)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Objects {
	Table,
	Updated,
}

