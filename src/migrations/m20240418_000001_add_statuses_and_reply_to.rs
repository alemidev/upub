use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Users::Table)
					.add_column(
						ColumnDef::new(Users::StatusesCount)
							.integer()
							.not_null()
							.default(0)
					)
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.add_column(
						ColumnDef::new(Objects::InReplyTo)
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
					.table(Users::Table)
					.drop_column(Users::StatusesCount)
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.drop_column(Objects::InReplyTo)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Users {
	Table,
	StatusesCount,
}

#[derive(DeriveIden)]
enum Objects {
	Table,
	InReplyTo,
}
