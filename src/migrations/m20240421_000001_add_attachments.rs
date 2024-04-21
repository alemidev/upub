use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Attachments::Table)
					.col(
						ColumnDef::new(Attachments::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Attachments::Url).string().not_null())
					.col(ColumnDef::new(Attachments::Object).string().not_null())
					.col(ColumnDef::new(Attachments::DocumentType).string().not_null())
					.col(ColumnDef::new(Attachments::Name).string().null())
					.col(ColumnDef::new(Attachments::MediaType).string().not_null())
					.col(ColumnDef::new(Attachments::Created).date_time().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("attachment-object-index")
					.table(Attachments::Table)
					.col(Attachments::Object)
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Attachments::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("attachment-object-index").to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum Attachments {
	Table,
	Id,
	Url,
	Object,
	DocumentType,
	Name,
	MediaType,
	Created,
}
