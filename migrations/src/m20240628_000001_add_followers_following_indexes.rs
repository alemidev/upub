use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::Actors;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_index(
				Index::create()
					.name("index-actors-followers")
					.table(Actors::Table)
					.col(Actors::Followers)
					.to_owned()
				)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-actors-following")
					.table(Actors::Table)
					.col(Actors::Following)
					.to_owned()
				)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("index-actors-followers").table(Actors::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-actors-following").table(Actors::Table).to_owned())
			.await?;

		Ok(())
	}
}
