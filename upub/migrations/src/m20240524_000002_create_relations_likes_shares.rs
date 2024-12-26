use sea_orm_migration::prelude::*;

use super::m20240524_000001_create_actor_activity_object_tables::{Activities, Actors, Objects};

#[derive(DeriveIden)]
pub enum Relations {
	Table,
	Internal,
	Follower,
	Following,
	Activity,
	Accept,
	FollowerInstance, // ADDED AFTERWARDS
	FollowingInstance, // ADDED AFTERWARDS
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub enum Likes {
	Table,
	Internal,
	Actor,
	Object,
	Activity, // DROPPED and ADDED BACK with migration m20241226_000002
	Content, // ADDED with migration m20241226_000002
	Published,
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub enum Announces {
	Table,
	Internal,
	Actor,
	Object,
	Activity, // ADDED with migration m20241226_000002
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
					.table(Relations::Table)
					.comment("follow relations between actors (applications too! for relays)")
					.col(
						ColumnDef::new(Relations::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Relations::Follower).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-follower")
							.from(Relations::Table, Relations::Follower)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Relations::Following).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-following")
							.from(Relations::Table, Relations::Following)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Relations::Accept).big_integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-accept")
							.from(Relations::Table, Relations::Accept)
							.to(Activities::Table, Activities::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Relations::Activity).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-activity")
							.from(Relations::Table, Relations::Activity)
							.to(Activities::Table, Activities::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-relations-follower").table(Relations::Table).col(Relations::Follower).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-relations-following").table(Relations::Table).col(Relations::Following).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-relations-activity").table(Relations::Table).col(Relations::Activity).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Likes::Table)
					.comment("all like events, joining actor to object")
					.col(
						ColumnDef::new(Likes::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Likes::Actor).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-likes-actor")
							.from(Likes::Table, Likes::Actor)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Likes::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-likes-object")
							.from(Likes::Table, Likes::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Likes::Activity).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-likes-activity")
							.from(Likes::Table, Likes::Activity)
							.to(Activities::Table, Activities::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Likes::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-likes-actor").table(Likes::Table).col(Likes::Actor).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-likes-object").table(Likes::Table).col(Likes::Object).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-likes-actor-object")
					.table(Likes::Table)
					.col(Likes::Actor)
					.col(Likes::Object)
					.to_owned()
			).await?;

		manager
			.create_table(
				Table::create()
					.table(Announces::Table)
					.comment("all share/boost/reblog events, joining actor to object")
					.col(
						ColumnDef::new(Announces::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Announces::Actor).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-announces-actor")
							.from(Announces::Table, Announces::Actor)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Announces::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-announces-object")
							.from(Announces::Table, Announces::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Announces::Published).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-announces-actor").table(Announces::Table).col(Announces::Actor).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-announces-object").table(Announces::Table).col(Announces::Object).to_owned())
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Relations::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Likes::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Announces::Table).to_owned())
			.await?;

		Ok(())
	}
}
