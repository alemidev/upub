use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::Objects;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_index(
				Index::create()
					.name("index-objects-audience")
					.table(Objects::Table)
					.col(Objects::Audience)
					.to_owned()
				)
			.await?;
		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("index-objects-audience").table(Objects::Table).to_owned())
			.await?;
		Ok(())
	}
}
