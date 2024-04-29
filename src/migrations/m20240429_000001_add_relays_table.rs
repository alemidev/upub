use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Relays::Table)
					.col(
						ColumnDef::new(Relays::Id)
							.string()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Relays::Accepted).boolean().not_null().default(false))
					.col(ColumnDef::new(Relays::Forwarding).boolean().not_null().default(false))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Relays::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Relays {
	Table,
	Id,
	Accepted,
	Forwarding,
}
