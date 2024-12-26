use sea_orm_migration::prelude::*;

use crate::{m20240524_000001_create_actor_activity_object_tables::Activities, m20240524_000002_create_relations_likes_shares::{Announces, Likes}};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Likes::Table)
					.add_column(ColumnDef::new(Likes::Content).string().not_null().default(""))
					.add_column(ColumnDef::new(Likes::Activity).big_integer().null())
					.add_foreign_key(
						TableForeignKey::new()
							.name("fkey-likes-activity")
							.from_tbl(Likes::Table)
							.from_col(Likes::Activity)
							.to_tbl(Activities::Table)
							.to_col(Activities::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.to_owned()
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("index-likes-actor-object")
					.table(Likes::Table)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-likes-actor-object-content")
					.table(Likes::Table)
					.col(Likes::Actor)
					.col(Likes::Object)
					.col(Likes::Content)
					.to_owned()
			).await?;

		manager
			.alter_table(
				Table::alter()
					.table(Announces::Table)
					.add_column(ColumnDef::new(Announces::Activity).big_integer().null())
					.add_foreign_key(
						TableForeignKey::new()
							.name("fkey-announces-activity")
							.from_tbl(Announces::Table)
							.from_col(Announces::Activity)
							.to_tbl(Activities::Table)
							.to_col(Activities::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(Likes::Table)
					.drop_column(Likes::Activity)
					.to_owned()
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("index-likes-actor-object-content")
					.table(Likes::Table)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-likes-actor-object")
					.table(Likes::Table)
					.col(Likes::Actor)
					.col(Likes::Object)
					.to_owned()
			).await?;

		manager
			.alter_table(
				Table::alter()
					.table(Announces::Table)
					.drop_column(Announces::Activity)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
