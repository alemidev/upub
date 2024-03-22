use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Relations::Table)
					.col(
						ColumnDef::new(Relations::Id)
							.integer()
							.auto_increment()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Relations::Follower).string().not_null())
					.col(ColumnDef::new(Relations::Following).string().not_null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Relations::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Relations {
	Table,
	Id,
	Follower,
	Following,
}
