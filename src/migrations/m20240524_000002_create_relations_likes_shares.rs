use sea_orm_migration::prelude::*;

use super::m20240524_000001_create_actor_activity_object_tables::{Activities, Actors, Objects};

#[derive(DeriveIden)]
pub enum Relations {
	Table,
	Id,
	Follower,
	Following,
	Activity,
	Accept,
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub enum Likes {
	Table,
	Id,
	Actor,
	Likes,
	Published,
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub enum Announces {
	Table,
	Id,
	Actor,
	Announces,
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
						ColumnDef::new(Relations::Id)
							.integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Relations::Follower).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-follower")
							.from(Relations::Table, Relations::Follower)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Relations::Following).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-following")
							.from(Relations::Table, Relations::Following)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Relations::Accept).integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-accept")
							.from(Relations::Table, Relations::Accept)
							.to(Activities::Table, Activities::Id)
							.on_update(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Relations::Activity).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-relations-activity")
							.from(Relations::Table, Relations::Activity)
							.to(Activities::Table, Activities::Id)
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
			.create_table(
				Table::create()
					.table(Likes::Table)
					.comment("all like events, joining actor to object")
					.col(
						ColumnDef::new(Likes::Id)
							.integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Likes::Actor).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-likes-actor")
							.from(Likes::Table, Likes::Actor)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Likes::Likes).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-likes-likes")
							.from(Likes::Table, Likes::Likes)
							.to(Objects::Table, Objects::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Likes::Published).date_time().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-likes-actor").table(Likes::Table).col(Likes::Actor).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-likes-likes").table(Likes::Table).col(Likes::Likes).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-likes-actor-likes")
					.table(Likes::Table)
					.col(Likes::Actor)
					.col(Likes::Likes)
					.to_owned()
			).await?;

		manager
			.create_table(
				Table::create()
					.table(Announces::Table)
					.comment("all share/boost/reblog events, joining actor to object")
					.col(
						ColumnDef::new(Announces::Id)
							.integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Announces::Actor).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-announces-actor")
							.from(Announces::Table, Announces::Actor)
							.to(Actors::Table, Actors::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Announces::Announces).integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-announces-announces")
							.from(Announces::Table, Announces::Announces)
							.to(Objects::Table, Objects::Id)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(Announces::Published).date_time().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-announces-actor").table(Announces::Table).col(Announces::Actor).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-announces-announces").table(Announces::Table).col(Announces::Announces).to_owned())
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Relations::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-relations-follower").table(Relations::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-relations-following").table(Relations::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Likes::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-likes-actor").table(Likes::Table).to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("index-likes-likes").table(Likes::Table).to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("index-likes-actor-likes").table(Likes::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Announces::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("shares-actor-index").table(Announces::Table).to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("shares-shares-index").table(Announces::Table).to_owned())
			.await?;

		Ok(())
	}
}
