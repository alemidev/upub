use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::{Activities, Actors};

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub enum Notifications {
	Table,
	Internal,
	Activity,
	Actor,
	Seen,
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
					.table(Notifications::Table)
					.comment("notifications table, connecting activities to users")
					.col(
						ColumnDef::new(Notifications::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Notifications::Actor).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-notifications-actor")
							.from(Notifications::Table, Notifications::Actor)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Notifications::Activity).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-notifications-activity")
							.from(Notifications::Table, Notifications::Activity)
							.to(Activities::Table, Activities::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Notifications::Seen).boolean().not_null().default(false))
					.col(ColumnDef::new(Notifications::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-notifications-activity").table(Notifications::Table).col(Notifications::Activity).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-notifications-actor-published")
					.table(Notifications::Table)
					.col(Notifications::Actor)
					.col((Notifications::Published, IndexOrder::Desc))
					.to_owned()
			).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Notifications::Table).to_owned())
			.await?;

		Ok(())
	}
}
