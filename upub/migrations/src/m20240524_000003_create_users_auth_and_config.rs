use sea_orm_migration::prelude::*;

use super::m20240524_000001_create_actor_activity_object_tables::Actors;

#[derive(DeriveIden)]
pub enum Configs {
	Table,
	Internal,
	Actor,
	AcceptFollowRequests,
	ShowFollowersCount,
	ShowFollowingCount,
	ShowFollowers,
	ShowFollowing,
}

#[derive(DeriveIden)]
pub enum Credentials {
	Table,
	Internal,
	Actor,
	Login,
	Password,
	Active, // ADDED
}

#[derive(DeriveIden)]
pub enum Sessions {
	Table,
	Internal,
	Actor,
	Secret,
	Expires,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Configs::Table)
					.comment("configuration for each local user")
					.col(
						ColumnDef::new(Configs::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Configs::Actor).string().not_null().unique_key())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-config-actor")
							.from(Configs::Table, Configs::Actor)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Configs::AcceptFollowRequests).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowersCount).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowingCount).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowers).boolean().not_null())
					.col(ColumnDef::new(Configs::ShowFollowing).boolean().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().unique().name("index-configs-actor").table(Configs::Table).col(Configs::Actor).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Credentials::Table)
					.comment("simple login credentials to authenticate local users")
					.col(
						ColumnDef::new(Credentials::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Credentials::Actor).string().not_null().unique_key())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-credentials-actor")
							.from(Credentials::Table, Credentials::Actor)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Credentials::Login).string().not_null())
					.col(ColumnDef::new(Credentials::Password).string().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().unique().name("index-credentials-actor").table(Credentials::Table).col(Credentials::Actor).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-credentials-login").table(Credentials::Table).col(Credentials::Login).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Sessions::Table)
					.comment("authenticated sessions from local users")
					.col(
						ColumnDef::new(Sessions::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Sessions::Actor).string().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-sessions-actor")
							.from(Sessions::Table, Sessions::Actor)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Sessions::Secret).string().not_null())
					.col(ColumnDef::new(Sessions::Expires).timestamp_with_time_zone().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-sessions-secret").table(Sessions::Table).col(Sessions::Secret).to_owned())
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Configs::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Credentials::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Sessions::Table).to_owned())
			.await?;

		Ok(())
	}
}
