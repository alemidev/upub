use sea_orm_migration::prelude::*;

use super::m20240524_000001_create_actor_activity_object_tables::{Activities, Actors, Instances, Objects};

#[derive(DeriveIden)]
pub enum Addressing {
	Table,
	Internal,
	Actor,
	Instance,
	Activity,
	Object,
	Published,
}

#[derive(DeriveIden)]
pub enum Deliveries {
	Table,
	Internal,
	Actor,
	Target,
	Activity,
	Published,
	NotBefore,
	Attempt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Addressing::Table)
					.comment("this join table contains all addressing relations, serving effectively as permissions truth table")
					.col(
						ColumnDef::new(Addressing::Internal)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Addressing::Actor).big_integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-addressing-actor")
							.from(Addressing::Table, Addressing::Actor)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Addressing::Instance).big_integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-addressing-instance")
							.from(Addressing::Table, Addressing::Instance)
							.to(Instances::Table, Instances::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::NoAction)
					)
					.col(ColumnDef::new(Addressing::Activity).big_integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-addressing-activity")
							.from(Addressing::Table, Addressing::Activity)
							.to(Activities::Table, Activities::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Addressing::Object).big_integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-addressing-object")
							.from(Addressing::Table, Addressing::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Addressing::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-addressing-actor-published")
					.table(Addressing::Table)
					.col(Addressing::Actor)
					.col(Addressing::Published)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-addressing-instance-published")
					.table(Addressing::Table)
					.col(Addressing::Instance)
					.col(Addressing::Published)
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-addressing-activity").table(Addressing::Table).col(Addressing::Activity).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-addressing-object").table(Addressing::Table).col(Addressing::Object).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Deliveries::Table)
					.comment("this table contains all enqueued outgoing delivery jobs")
					.col(
						ColumnDef::new(Deliveries::Internal)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Deliveries::Actor).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-deliveries-actor")
							.from(Deliveries::Table, Deliveries::Actor)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Deliveries::Target).string().not_null())
					.col(ColumnDef::new(Deliveries::Activity).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-deliveries-activity")
							.from(Deliveries::Table, Deliveries::Activity)
							.to(Activities::Table, Activities::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Deliveries::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Deliveries::NotBefore).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Deliveries::Attempt).integer().not_null().default(0))
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-deliveries-not-before")
					.table(Deliveries::Table)
					.col((Deliveries::NotBefore, IndexOrder::Asc))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Addressing::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Deliveries::Table).to_owned())
			.await?;

		Ok(())
	}
}
