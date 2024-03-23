use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_index(
				Index::create()
					.name("user-domain-index")
					.table(Users::Table)
					.col(Users::Domain)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("activities-published-descending-index")
					.table(Activities::Table)
					.col((Activities::Published, IndexOrder::Desc))
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("activities-actor-index")
					.table(Activities::Table)
					.col(Activities::Actor)
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("activities-object-index")
					.table(Activities::Table)
					.col(Activities::Object)
					.to_owned()
			).await?;

		manager
			.create_index(
				Index::create()
					.name("objects-attributed-to-index")
					.table(Objects::Table)
					.col(Objects::AttributedTo)
					.to_owned()
			).await?;

		manager
			.create_index(
				Index::create()
					.name("shares-actor-index")
					.table(Shares::Table)
					.col(Shares::Actor)
					.to_owned()
			).await?;

		manager
			.create_index(
				Index::create()
					.name("shares-shares-index")
					.table(Shares::Table)
					.col(Shares::Shares)
					.to_owned()
			).await?;

		manager
			.create_index(
				Index::create()
					.name("likes-actor-index")
					.table(Likes::Table)
					.col(Likes::Actor)
					.to_owned()
			).await?;

		manager
			.create_index(
				Index::create()
					.name("likes-likes-index")
					.table(Likes::Table)
					.col(Likes::Likes)
					.to_owned()
			).await?;

		manager
			.create_index(
				Index::create()
					.name("likes-actor-likes-index")
					.table(Likes::Table)
					.col(Likes::Actor)
					.col(Likes::Likes)
					.unique()
					.to_owned()
			).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_index(Index::drop().name("user-domain-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("activities-published-descending-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("activities-actor-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("activities-object-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("objects-attributed-to-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("shares-actor-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("shares-shares-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("likes-actor-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("likes-likes-index").to_owned())
			.await?;
		manager
			.drop_index(Index::drop().name("likes-actor-likes-index").to_owned())
			.await?;
		Ok(())
	}
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
enum Likes {
	Table,
	Actor,
	Likes,
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
enum Shares {
	Table,
	Actor,
	Shares,
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Domain,
}

#[derive(DeriveIden)]
enum Activities {
	Table,
	Actor,
	Object,
	Published,
}

#[derive(DeriveIden)]
enum Objects {
	Table,
	AttributedTo,
}
