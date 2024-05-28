use sea_orm_migration::prelude::*;
#[derive(DeriveIden)]
pub enum Actors {
	Table,
	Internal,
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
	FollowingCount,
	Followers,
	FollowersCount,
	StatusesCount,
	PublicKey,
	PrivateKey,
	Published,
	Updated,
}

#[derive(DeriveIden)]
pub enum Activities {
	Table,
	Internal,
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
pub enum Objects {
	Table,
	Internal,
	Id,
	ObjectType,
	AttributedTo,
	Name,
	Summary,
	Content,
	Sensitive,
	Url,
	Likes,
	Announces,
	Replies,
	Context,
	InReplyTo,
	Cc,
	Bcc,
	To,
	Bto,
	Published,
	Updated,
}

#[derive(DeriveIden)]
pub enum Instances {
	Table,
	Internal,
	Domain,
	Name,
	Software,
	Version,
	Icon,
	DownSince,
	Users,
	Posts,
	Published,
	Updated,
}


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.create_table(
				Table::create()
					.table(Instances::Table)
					.comment("known other instances in the fediverse")
					.col(
						ColumnDef::new(Instances::Internal)
							.big_integer()
							.not_null()
							.auto_increment()
							.primary_key()
					)
					.col(ColumnDef::new(Instances::Domain).string().not_null().unique_key())
					.col(ColumnDef::new(Instances::Name).string().null())
					.col(ColumnDef::new(Instances::Software).string().null())
					.col(ColumnDef::new(Instances::Version).string().null())
					.col(ColumnDef::new(Instances::Icon).string().null())
					.col(ColumnDef::new(Instances::DownSince).date_time().null())
					.col(ColumnDef::new(Instances::Users).integer().null())
					.col(ColumnDef::new(Instances::Posts).integer().null())
					.col(ColumnDef::new(Instances::Published).date_time().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Instances::Updated).date_time().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().unique().name("index-instances-domain").table(Instances::Table).col(Instances::Domain).to_owned())
			.await?;



		manager
			.create_table(
				Table::create()
					.table(Actors::Table)
					.comment("main actors table, with users and applications")
					.col(
						ColumnDef::new(Actors::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Actors::Id).string().not_null().unique_key())
					.col(ColumnDef::new(Actors::ActorType).string().not_null())
					.col(ColumnDef::new(Actors::Domain).string().not_null())
					// .foreign_key(
					// 	ForeignKey::create()
					// 		.name("fkey-actors-instances")
					// 		.from(Actors::Table, Actors::Domain)
					// 		.to(Instances::Table, Instances::Domain)
					// 		.on_update(ForeignKeyAction::Cascade)
					// )
					.col(ColumnDef::new(Actors::Name).string().null())
					.col(ColumnDef::new(Actors::Summary).string().null())
					.col(ColumnDef::new(Actors::Image).string().null())
					.col(ColumnDef::new(Actors::Icon).string().null())
					.col(ColumnDef::new(Actors::PreferredUsername).string().not_null())
					.col(ColumnDef::new(Actors::Inbox).string().null())
					.col(ColumnDef::new(Actors::SharedInbox).string().null())
					.col(ColumnDef::new(Actors::Outbox).string().null())
					.col(ColumnDef::new(Actors::Following).string().null())
					.col(ColumnDef::new(Actors::Followers).string().null())
					.col(ColumnDef::new(Actors::FollowingCount).integer().not_null().default(0))
					.col(ColumnDef::new(Actors::FollowersCount).integer().not_null().default(0))
					.col(ColumnDef::new(Actors::StatusesCount).integer().not_null().default(0))
					.col(ColumnDef::new(Actors::PublicKey).string().not_null())
					.col(ColumnDef::new(Actors::PrivateKey).string().null())
					.col(ColumnDef::new(Actors::Published).date_time().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Actors::Updated).date_time().not_null().default(Expr::current_timestamp()))
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().unique().name("index-actors-id").table(Actors::Table).col(Actors::Id).to_owned())
			.await?;

		manager
			.create_index(
				Index::create()
					.unique()
					.name("index-actors-preferred-username-domain")
					.table(Actors::Table)
					.col(Actors::PreferredUsername)
					.col(Actors::Domain)
					.to_owned()
				)
			.await?;

		manager
			.create_index(Index::create().name("index-actors-domain").table(Actors::Table).col(Actors::Domain).to_owned())
			.await?;



		manager
			.create_table(
				Table::create()
					.table(Objects::Table)
					.comment("objects are all AP documents which are neither actors nor activities")
					.col(
						ColumnDef::new(Objects::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Objects::Id).string().not_null().unique_key())
					.col(ColumnDef::new(Objects::ObjectType).string().not_null())
					.col(ColumnDef::new(Objects::AttributedTo).string().null())
					// .foreign_key(
					// 	ForeignKey::create()
					// 		.name("fkey-objects-attributed-to")
					// 		.from(Objects::Table, Objects::AttributedTo)
					// 		.to(Actors::Table, Actors::Internal)
					// 		.on_update(ForeignKeyAction::Cascade)
					// )
					.col(ColumnDef::new(Objects::Name).string().null())
					.col(ColumnDef::new(Objects::Summary).string().null())
					.col(ColumnDef::new(Objects::Content).string().null())
					.col(ColumnDef::new(Objects::Sensitive).boolean().not_null().default(false))
					.col(ColumnDef::new(Objects::InReplyTo).string().null())
					// .foreign_key(
					// 	ForeignKey::create()
					// 		.name("fkey-objects-in-reply-to")
					// 		.from(Objects::Table, Objects::InReplyTo)
					// 		.to(Objects::Table, Objects::Id)
					// 		.on_update(ForeignKeyAction::Cascade)
					// )
					.col(ColumnDef::new(Objects::Url).string().null())
					.col(ColumnDef::new(Objects::Likes).integer().not_null().default(0))
					.col(ColumnDef::new(Objects::Announces).integer().not_null().default(0))
					.col(ColumnDef::new(Objects::Replies).integer().not_null().default(0))
					.col(ColumnDef::new(Objects::Context).string().null())
					.col(ColumnDef::new(Objects::To).json().null())
					.col(ColumnDef::new(Objects::Bto).json().null())
					.col(ColumnDef::new(Objects::Cc).json().null())
					.col(ColumnDef::new(Objects::Bcc).json().null())
					.col(ColumnDef::new(Objects::Published).date_time().not_null().default(Expr::current_timestamp()))
					.col(ColumnDef::new(Objects::Updated).date_time().not_null().default(Expr::current_timestamp()))
					.to_owned()
			).await?;

		manager
			.create_index(Index::create().unique().name("index-objects-id").table(Objects::Table).col(Objects::Id).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-objects-attributed-to").table(Objects::Table).col(Objects::AttributedTo).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-objects-in-reply-to").table(Objects::Table).col(Objects::InReplyTo).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-objects-content-text").table(Objects::Table).col(Objects::Content).full_text().to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-objects-context").table(Objects::Table).col(Objects::Context).to_owned())
			.await?;



		manager
			.create_table(
				Table::create()
					.table(Activities::Table)
					.comment("all activities this instance ever received or generated")
					.col(
						ColumnDef::new(Activities::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(Activities::Id).string().not_null().unique_key())
					.col(ColumnDef::new(Activities::ActivityType).string().not_null())
					.col(ColumnDef::new(Activities::Actor).string().not_null())
					// .foreign_key(
					// 	ForeignKey::create()
					// 		.name("fkey-activities-actors")
					// 		.from(Activities::Table, Activities::Actor)
					// 		.to(Actors::Table, Actors::Id)
					// 		.on_update(ForeignKeyAction::Cascade)
					// )
					.col(ColumnDef::new(Activities::Object).string().null())
					// .foreign_key(
					// 	ForeignKey::create()
					// 		.name("fkey-activities-objects")
					// 		.from(Activities::Table, Activities::Object)
					// 		.to(Objects::Table, Objects::Internal)
					// 		.on_update(ForeignKeyAction::Cascade)
					// )
					.col(ColumnDef::new(Activities::Target).string().null())
					.col(ColumnDef::new(Activities::To).json().null())
					.col(ColumnDef::new(Activities::Bto).json().null())
					.col(ColumnDef::new(Activities::Cc).json().null())
					.col(ColumnDef::new(Activities::Bcc).json().null())
					.col(ColumnDef::new(Activities::Published).date_time().not_null().default(Expr::current_timestamp()))
					.to_owned()
			).await?;

		manager
			.create_index(Index::create().unique().name("index-activities-id").table(Activities::Table).col(Activities::Id).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-activities-actor").table(Activities::Table).col(Activities::Actor).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("activities-object-index").table(Activities::Table).col(Activities::Object).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-activities-published-descending").table(Activities::Table).col((Activities::Published, IndexOrder::Desc)).to_owned())
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Actors::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-actors-id").table(Actors::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-actors-preferred-username").table(Actors::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-actors-domain").table(Actors::Table).to_owned())
			.await?;


		manager
			.drop_table(Table::drop().table(Activities::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-activities-id").table(Activities::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-activities-actor").table(Activities::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("activities-object-index").table(Activities::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-activities-published-descending").table(Activities::Table).to_owned())
			.await?;


		manager
			.drop_table(Table::drop().table(Objects::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-objects-id").table(Objects::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-objects-attributed-to").table(Objects::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-objects-in-reply-to").table(Objects::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-objects-content-text").table(Objects::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-objects-context").table(Objects::Table).to_owned())
			.await?;


		manager
			.drop_table(Table::drop().table(Instances::Table).to_owned())
			.await?;

		manager
			.drop_index(Index::drop().name("index-instances-domain").table(Instances::Table).to_owned())
			.await?;

		Ok(())
	}
}
