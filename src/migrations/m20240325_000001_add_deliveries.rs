use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Deliveries::Table)
					.col(
						ColumnDef::new(Deliveries::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Deliveries::Actor).string().not_null())
					.col(ColumnDef::new(Deliveries::Target).string().not_null())
					.col(ColumnDef::new(Deliveries::Activity).string().not_null())
					.col(ColumnDef::new(Deliveries::Created).date_time().not_null())
					.col(ColumnDef::new(Deliveries::NotBefore).date_time().not_null())
					.col(ColumnDef::new(Deliveries::Attempt).integer().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("deliveries-notbefore-index")
					.table(Deliveries::Table)
					.col((Deliveries::NotBefore, IndexOrder::Asc))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Deliveries::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("deliveries-notbefore-index").to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Deliveries {
	Table,
	Id,
	Actor,
	Target,
	Activity,
	Created,
	NotBefore,
	Attempt,
}
