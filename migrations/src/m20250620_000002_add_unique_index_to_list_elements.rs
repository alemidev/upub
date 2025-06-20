use sea_orm_migration::prelude::*;

use crate::m20250620_000001_add_list_tables::ListElements;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_index(
				Index::create()
					.name("index-list-elements-list-actor-object-unique")
					.table(ListElements::Table)
					.col(ListElements::List)
					.col(ListElements::Actor)
					.col(ListElements::Object)
					.unique()
					.to_owned()
				)
			.await?;
		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("index-list-elements-list-actor-object-unique").table(ListElements::Table).to_owned())
			.await?;
		Ok(())
	}
}
