use sea_orm_migration::prelude::*;

use super::m20240524_000001_create_actor_activity_object_tables::{Actors, Objects};

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub enum Dislikes {
	Table,
	Internal,
	Actor,
	Object,
	Published,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Dislikes::Table)
					.comment("all dislike events, joining actor to object")
					.col(
						ColumnDef::new(Dislikes::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Dislikes::Actor).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-dislikes-actor")
							.from(Dislikes::Table, Dislikes::Actor)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Dislikes::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-dislikes-object")
							.from(Dislikes::Table, Dislikes::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Dislikes::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-dislikes-actor").table(Dislikes::Table).col(Dislikes::Actor).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-dislikes-object").table(Dislikes::Table).col(Dislikes::Object).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-dislikes-actor-object")
					.table(Dislikes::Table)
					.col(Dislikes::Actor)
					.col(Dislikes::Object)
					.to_owned()
			).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Dislikes::Table).to_owned())
			.await?;

		Ok(())
	}
}
