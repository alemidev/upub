use sea_orm_migration::prelude::*;

use super::m20240524_000001_create_actor_activity_object_tables::Objects;

#[derive(DeriveIden)]
pub enum Attachments {
	Table,
	Internal,
	DocumentType,
	Url,
	Object,
	Name,
	MediaType,
}

#[derive(DeriveIden)]
pub enum Mentions {
	Table,
	Internal,
	Object,
	Actor,
}

#[derive(DeriveIden)]
pub enum Hashtags {
	Table,
	Internal,
	Object,
	Name,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Attachments::Table)
					.comment("media attachments related to objects")
					.col(
						ColumnDef::new(Attachments::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Attachments::Url).string().not_null())
					.col(ColumnDef::new(Attachments::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-attachments-object")
							.from(Attachments::Table, Attachments::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Attachments::DocumentType).string().not_null())
					.col(ColumnDef::new(Attachments::Name).string().null())
					.col(ColumnDef::new(Attachments::MediaType).string().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-attachment-object").table(Attachments::Table).col(Attachments::Object).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-attachment-url").table(Attachments::Table).col(Attachments::Url).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Mentions::Table)
					.comment("join table relating posts to mentioned users")
					.col(
						ColumnDef::new(Mentions::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Mentions::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-mentions-object")
							.from(Mentions::Table, Mentions::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Mentions::Actor).string().not_null())
					// .foreign_key(
					// 	ForeignKey::create()
					// 		.name("fkey-mentions-actor")
					// 		.from(Mentions::Table, Mentions::Actor)
					// 		.to(Actors::Table, Actors::Internal)
					// 		.on_update(ForeignKeyAction::Cascade)
					// 		.on_delete(ForeignKeyAction::Cascade)
					// )
					.col(ColumnDef::new(Mentions::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-mentions-object").table(Mentions::Table).col(Mentions::Object).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-mentions-actor").table(Mentions::Table).col(Mentions::Actor).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Hashtags::Table)
					.comment("join table relating posts to hashtags")
					.col(
						ColumnDef::new(Hashtags::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Hashtags::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-hashtags-object")
							.from(Hashtags::Table, Hashtags::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Hashtags::Name).string().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-hashtags-object").table(Hashtags::Table).col(Hashtags::Object).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-hashtags-name").table(Hashtags::Table).col(Hashtags::Name).to_owned())
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Attachments::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Mentions::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Hashtags::Table).to_owned())
			.await?;

		Ok(())
	}
}
