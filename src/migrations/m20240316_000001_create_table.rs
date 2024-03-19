use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Users::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Users::Id)
							.string()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Users::ActorType).string().not_null())
					.col(ColumnDef::new(Users::Name).string().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Activities::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Activities::Id)
							.string()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Activities::ActivityType).string().not_null())
					.col(ColumnDef::new(Activities::Actor).string().not_null())
					.col(ColumnDef::new(Activities::Object).string().null())
					.col(ColumnDef::new(Activities::Target).string().null())
					.col(ColumnDef::new(Activities::Published).string().null())
					.to_owned()
			).await?;

		manager
			.create_table(
				Table::create()
					.table(Objects::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Objects::Id)
							.string()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Objects::ObjectType).string().not_null())
					.col(ColumnDef::new(Objects::AttributedTo).string().null())
					.col(ColumnDef::new(Objects::Name).string().null())
					.col(ColumnDef::new(Objects::Summary).string().null())
					.col(ColumnDef::new(Objects::Content).string().null())
					.col(ColumnDef::new(Objects::Published).string().null())
					.to_owned()
			).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Users::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Activities::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Objects::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
	ActorType,
	Name,
}

#[derive(DeriveIden)]
enum Activities {
	Table,
	Id,
	ActivityType,
	Actor,
	Object,
	Target,
	Published
}

#[derive(DeriveIden)]
enum Objects {
	Table,
	Id,
	ObjectType,
	Name,
	Summary,
	AttributedTo,
	Content,
	Published,
}
