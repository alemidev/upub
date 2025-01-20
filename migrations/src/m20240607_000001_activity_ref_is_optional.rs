use sea_orm_migration::prelude::*;

use crate::m20240524_000002_create_relations_likes_shares::{Announces, Likes};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(Likes::Table)
					.drop_column(Likes::Activity)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-announces-actor-object")
					.table(Announces::Table)
					.col(Announces::Actor)
					.col(Announces::Object)
					.to_owned()
			).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(Likes::Table)
					.add_column(ColumnDef::new(Likes::Activity).big_integer().not_null())
					.to_owned()
			)
			.await?;

		manager
			.drop_index(Index::drop().name("index-announces-actor-object").table(Announces::Table).to_owned())
			.await?;

		Ok(())
	}
}
