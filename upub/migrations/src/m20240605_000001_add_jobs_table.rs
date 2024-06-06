use sea_orm_migration::prelude::*;

use crate::{m20240524_000001_create_actor_activity_object_tables::{Activities, Actors}, m20240524_000004_create_addressing_deliveries::Deliveries};

#[derive(DeriveIden)]
pub enum Jobs {
	Table,
	Internal,
	JobType,
	Actor,
	Target,
	Activity,
	Payload,
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
			.drop_table(Table::drop().table(Deliveries::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-deliveries-not-before").table(Deliveries::Table).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Jobs::Table)
					.comment("background job queue: delivery, fetch and processing tasks")
					.col(
						ColumnDef::new(Jobs::Internal)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Jobs::JobType).small_integer().not_null())
					.col(ColumnDef::new(Jobs::Actor).string().not_null())
					.col(ColumnDef::new(Jobs::Target).string().null())
					.col(ColumnDef::new(Jobs::Activity).string().not_null().unique_key())
					.col(ColumnDef::new(Jobs::Payload).string().null())
					.col(ColumnDef::new(Jobs::Published).date_time().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Jobs::NotBefore).date_time().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Jobs::Attempt).small_integer().not_null().default(0))
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-jobs-activity")
					.table(Jobs::Table)
					.col(Jobs::Activity)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-jobs-not-before")
					.table(Jobs::Table)
					.col((Jobs::NotBefore, IndexOrder::Asc))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Jobs::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-jobs-activity").table(Jobs::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-jobs-not-before").table(Jobs::Table).to_owned())
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
					.col(ColumnDef::new(Deliveries::Published).date_time().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Deliveries::NotBefore).date_time().not_null().default(Expr::current_timestamp()))
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
}
