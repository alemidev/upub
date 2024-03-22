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
					.col(
						ColumnDef::new(Users::Id)
							.string()
							.not_null()
							.primary_key()
					)
					.col(ColumnDef::new(Users::ActorType).string().not_null())
					.col(ColumnDef::new(Users::Domain).string().not_null())
					.col(ColumnDef::new(Users::Name).string().not_null())
					.col(ColumnDef::new(Users::Summary).string().null())
					.col(ColumnDef::new(Users::Image).string().null())
					.col(ColumnDef::new(Users::Icon).string().null())
					.col(ColumnDef::new(Users::PreferredUsername).string().null())
					.col(ColumnDef::new(Users::Inbox).string().null())
					.col(ColumnDef::new(Users::SharedInbox).string().null())
					.col(ColumnDef::new(Users::Outbox).string().null())
					.col(ColumnDef::new(Users::Following).string().null())
					.col(ColumnDef::new(Users::Followers).string().null())
					.col(ColumnDef::new(Users::PublicKey).string().not_null())
					.col(ColumnDef::new(Users::PrivateKey).string().null())
					.col(ColumnDef::new(Users::Created).date_time().not_null())
					.col(ColumnDef::new(Users::Updated).date_time().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Activities::Table)
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
					.col(ColumnDef::new(Activities::To).json().null())
					.col(ColumnDef::new(Activities::Bto).json().null())
					.col(ColumnDef::new(Activities::Cc).json().null())
					.col(ColumnDef::new(Activities::Bcc).json().null())
					.col(ColumnDef::new(Activities::Published).date_time().not_null())
					.to_owned()
			).await?;

		manager
			.create_table(
				Table::create()
					.table(Objects::Table)
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
					.col(ColumnDef::new(Objects::Likes).integer().not_null().default(0))
					.col(ColumnDef::new(Objects::Shares).integer().not_null().default(0))
					.col(ColumnDef::new(Objects::Comments).integer().not_null().default(0))
					.col(ColumnDef::new(Objects::Context).string().null())
					.col(ColumnDef::new(Objects::To).json().null())
					.col(ColumnDef::new(Objects::Bto).json().null())
					.col(ColumnDef::new(Objects::Cc).json().null())
					.col(ColumnDef::new(Objects::Bcc).json().null())
					.col(ColumnDef::new(Objects::Published).string().not_null())
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
	Domain,
	ActorType,
	Name,
	Summary,
	Image,
	Icon,
	PreferredUsername,
	Inbox,
	SharedInbox,
	Outbox,
	Following,
	Followers,
	PublicKey,
	PrivateKey,
	Created,
	Updated,
}

#[derive(DeriveIden)]
enum Activities {
	Table,
	Id,
	ActivityType,
	Actor,
	Object,
	Target,
	Cc,
	Bcc,
	To,
	Bto,
	Published,
}

#[derive(DeriveIden)]
enum Objects {
	Table,
	Id,
	ObjectType,
	AttributedTo,
	Name,
	Summary,
	Content,
	Likes,
	Shares,
	Comments,
	Context,
	Cc,
	Bcc,
	To,
	Bto,
	Published,
}
