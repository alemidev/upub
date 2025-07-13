use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::Instances;

#[derive(DeriveIden)]
pub enum Emojis {
	Table,
	Internal,
	Name,
	Domain,
	Uri,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.create_table(
				Table::create()
					.table(Emojis::Table)
					.comment("custom emojis discovered, with their urls")
					.col(
						ColumnDef::new(Emojis::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Emojis::Name).string().not_null())
					.col(ColumnDef::new(Emojis::Domain).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-emojis-instances")
							.from(Emojis::Table, Emojis::Domain)
							.to(Instances::Table, Instances::Domain)
							.on_update(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Emojis::Uri).string().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("index-emojis-name-domain")
					.table(Emojis::Table)
					.col(Emojis::Name)
					.col(Emojis::Domain)
					.unique()
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Emojis::Table).to_owned())
			.await?;

		Ok(())
	}
}
