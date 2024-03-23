use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Configs::Table)
					.col(
						ColumnDef::new(Configs::Id)
							.string()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Configs::AcceptFollowRequests).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowersCount).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowingCount).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowers).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowing).boolean().not_null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Configs::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Configs {
	Table,
	Id,
	AcceptFollowRequests,
	ShowFollowersCount,
	ShowFollowingCount,
	ShowFollowers,
	ShowFollowing,
}
