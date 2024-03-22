use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Likes::Table)
					.col(
						ColumnDef::new(Likes::Id)
							.integer()
							.auto_increment()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Likes::Actor).string().not_null())
					.col(ColumnDef::new(Likes::Likes).string().not_null())
					.index(Index::create().col(Likes::Actor).col(Likes::Likes).unique())
					.index(Index::create().col(Likes::Actor))
					.index(Index::create().col(Likes::Likes))
					.to_owned()
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Shares::Table)
					.col(
						ColumnDef::new(Shares::Id)
							.integer()
							.auto_increment()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Shares::Actor).string().not_null())
					.col(ColumnDef::new(Shares::Shares).string().not_null())
					.col(ColumnDef::new(Shares::Date).date_time().not_null())
					.index(Index::create().col(Shares::Actor))
					.index(Index::create().col(Shares::Shares))
					.to_owned()
			)
			.await?;


		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Likes::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Shares::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Likes {
	Table,
	Id,
	Actor,
	Likes,
}

#[derive(DeriveIden)]
enum Shares {
	Table,
	Id,
	Actor,
	Shares,
	Date,
}
