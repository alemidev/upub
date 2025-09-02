use sea_orm_migration::prelude::*;

use crate::m20240524_000004_create_addressing_deliveries::Addressing;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("index-addressing-actor-published").table(Addressing::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-addressing-instance-published").table(Addressing::Table).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-addressing-actor")
					.table(Addressing::Table)
					.col(Addressing::Actor)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-addressing-instance")
					.table(Addressing::Table)
					.col(Addressing::Instance)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-addressing-published")
					.table(Addressing::Table)
					.col(Addressing::Published)
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("index-addressing-published").table(Addressing::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-addressing-actor").table(Addressing::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-addressing-instance").table(Addressing::Table).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-addressing-actor-published")
					.table(Addressing::Table)
					.col(Addressing::Actor)
					.col(Addressing::Published)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-addressing-instance-published")
					.table(Addressing::Table)
					.col(Addressing::Instance)
					.col(Addressing::Published)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
