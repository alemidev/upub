use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::Instances;

#[derive(DeriveIden)]
pub enum Downtimes {
	Table,
	Internal,
	Domain,
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
					.table(Downtimes::Table)
					.comment("tracking remote instances downtimes")
					.col(
						ColumnDef::new(Downtimes::Internal)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Downtimes::Domain).string().not_null().unique_key())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-downtime-instances")
							.from(Downtimes::Table, Downtimes::Domain)
							.to(Instances::Table, Instances::Domain)
							.on_update(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Downtimes::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().unique().name("index-downtimes-domain").table(Downtimes::Table).col(Downtimes::Domain).to_owned())
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Downtimes::Table).to_owned())
			.await?;

		Ok(())
	}
}
