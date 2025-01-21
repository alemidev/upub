use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::Actors;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Actors::Table)
					.add_column(ColumnDef::new(Actors::MovedTo).string().null())
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Actors::Table)
					.add_column(ColumnDef::new(Actors::AlsoKnownAs).json_binary().null())
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Actors::Table)
					.add_column(ColumnDef::new(Actors::Fields).json_binary().null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Actors::Table)
					.drop_column(Actors::MovedTo)
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Actors::Table)
					.drop_column(Actors::AlsoKnownAs)
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Actors::Table)
					.drop_column(Actors::Fields)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
