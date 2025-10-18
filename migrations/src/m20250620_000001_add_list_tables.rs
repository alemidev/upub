use sea_orm_migration::prelude::*;

use crate::{m20240524_000001_create_actor_activity_object_tables::{Actors, Objects}, m20240524_000003_create_users_auth_and_config::Configs};

#[derive(DeriveIden)]
pub enum Lists {
	Table,
	Internal,
	Id,
	AttributedTo,
	Name,
	Summary,
	Published,
	Updated,
}

#[derive(DeriveIden)]
pub enum ListElements {
	Table,
	Internal,
	List,
	Object,
	Actor,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.create_table(
				Table::create()
					.table(Lists::Table)
					.comment("user-managed collections of objects or actors")
					.col(
						ColumnDef::new(Lists::Internal)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Lists::Id).string().not_null())
					.col(ColumnDef::new(Lists::AttributedTo).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-lists-actors")
							.from(Lists::Table, Lists::AttributedTo)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Lists::Name).string().not_null())
					.col(ColumnDef::new(Lists::Summary).string().null())
					.col(ColumnDef::new(Lists::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Lists::Updated).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().unique().name("index-lists-id").table(Lists::Table).col(Lists::Id).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-lists-actor").table(Lists::Table).col(Lists::AttributedTo).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(ListElements::Table)
					.comment("elements belonging in a list")
					.col(
						ColumnDef::new(ListElements::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(ListElements::List).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-list-element-list")
							.from(ListElements::Table, ListElements::List)
							.to(Lists::Table, Lists::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(ListElements::Object).big_integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-list-element-object")
							.from(ListElements::Table, ListElements::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(ListElements::Actor).big_integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-list-element-actor")
							.from(ListElements::Table, ListElements::Actor)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-list-element-list").table(ListElements::Table).col(ListElements::List).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Configs::Table)
					.add_column(ColumnDef::new(Configs::ShowLists).boolean().not_null().default(false))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Lists::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(ListElements::Table).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Configs::Table)
					.drop_column(Configs::ShowLists)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
