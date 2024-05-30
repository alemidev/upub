use sea_orm_migration::prelude::*;

use super::m20240524_000002_create_relations_likes_shares::Relations;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-relations-follower-following")
					.table(Relations::Table)
					.col(Relations::Following)
					.col(Relations::Follower)
					.to_owned()
				)
			.await?;
		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("index-relations-follower-following").table(Relations::Table).to_owned())
			.await?;
		Ok(())
	}
}
