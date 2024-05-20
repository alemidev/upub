use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.drop_index(Index::drop().name("addressing-actor-index").to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("addressing-actor-published-index")
					.table(Addressing::Table)
					.col(Addressing::Actor)
					.col(Addressing::Published)
					.to_owned()
			)
			.await?;

		manager
			.drop_index(Index::drop().name("addressing-server-index").to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("addressing-server-published-index")
					.table(Addressing::Table)
					.col(Addressing::Server)
					.col(Addressing::Published)
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("addressing-actor-published-index").to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("addressing-actor-index")
					.table(Addressing::Table)
					.col(Addressing::Actor)
					.to_owned()
			)
			.await?;

		manager
			.drop_index(Index::drop().name("addressing-server-published-index").to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("addressing-server-index")
					.table(Addressing::Table)
					.col(Addressing::Server)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Addressing {
	Table,
	Actor,
	Server,
	Published,
}
