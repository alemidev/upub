use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Addressing::Table)
					.col(
						ColumnDef::new(Addressing::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Addressing::Actor).string().not_null())
					.col(ColumnDef::new(Addressing::Server).string().not_null())
					.col(ColumnDef::new(Addressing::Activity).string().not_null())
					.col(ColumnDef::new(Addressing::Object).string().null())
					.col(ColumnDef::new(Addressing::Published).date_time().not_null())
					.to_owned()
			)
			.await?;

		// TODO these indexes may not be ordered, killing out timeline query performance
		//      it may be necessary to include datetime in the index itself? or maybe specify
		//      some ordering to use another type of indes?

		manager
			.create_index(
				Index::create()
					.name("addressing-actor-index")
					.table(Addressing::Table)
					.col(Addressing::Actor)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("addressing-server-index")
					.table(Addressing::Table)
					.col(Addressing::Server)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("addressing-activity-index")
					.table(Addressing::Table)
					.col(Addressing::Activity)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("addressing-object-index")
					.table(Addressing::Table)
					.col(Addressing::Object)
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
			.drop_index(Index::drop().name("addressing-actor-index").to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("addressing-server-index").to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("addressing-activity-index").to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("addressing-object-index").to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Addressing {
	Table,
	Id,
	Actor,
	Server,
	Activity,
	Object,
	Published,
}
