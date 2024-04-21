use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Addressing::Table)
					.modify_column(
						ColumnDef::new(Addressing::Activity)
							.string()
							.null()
					)
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Addressing::Table)
					.modify_column(
						ColumnDef::new(Addressing::Activity)
							.string()
							.not_null()
							.default("")
					)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Addressing {
	Table,
	Activity,
}
