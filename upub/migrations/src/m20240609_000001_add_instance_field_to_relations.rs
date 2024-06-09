use sea_orm_migration::prelude::*;

use crate::{m20240524_000001_create_actor_activity_object_tables::Instances, m20240524_000002_create_relations_likes_shares::Relations};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(Relations::Table)
					.add_column(ColumnDef::new(Relations::FollowerInstance).big_integer().not_null())
					.add_foreign_key(
						TableForeignKey::new()
							.name("fkey-relations-follower-instance")
							.from_tbl(Relations::Table)
							.from_col(Relations::FollowerInstance)
							.to_tbl(Instances::Table)
							.to_col(Instances::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.add_column(ColumnDef::new(Relations::FollowingInstance).big_integer().not_null())
					.add_foreign_key(
						TableForeignKey::new()
							.name("fkey-relations-following-instance")
							.from_tbl(Relations::Table)
							.from_col(Relations::FollowingInstance)
							.to_tbl(Instances::Table)
							.to_col(Instances::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-relations-follower-instance")
					.table(Relations::Table)
					.col(Relations::FollowerInstance)
					.to_owned()
			).await?;

		manager
			.create_index(
				Index::create()
					.name("index-relations-following-instance")
					.table(Relations::Table)
					.col(Relations::FollowingInstance)
					.to_owned()
			).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.drop_index(
				Index::drop()
					.name("index-relations-follower-instance")
					.table(Relations::Table)
					.to_owned()
			)
			.await?;

		manager
			.drop_index(
				Index::drop()
					.name("index-relations-following-instance")
					.table(Relations::Table)
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Relations::Table)
					.drop_foreign_key(Alias::new("fkey-relations-follower-instance"))
					.drop_column(Relations::FollowerInstance)
					.drop_foreign_key(Alias::new("fkey-relations-following-instance"))
					.drop_column(Relations::FollowingInstance)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

