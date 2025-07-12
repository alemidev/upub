use sea_orm_migration::prelude::*;

use crate::m20240524_000001_create_actor_activity_object_tables::{Actors, Objects};

#[derive(DeriveIden)]
pub enum QuestionOptions {
	Table,
	Internal,
	Object,
	Name,
}

#[derive(DeriveIden)]
pub enum QuestionAnswers {
	Table,
	Internal,
	Object,
	Actor,
	Answer,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.create_table(
				Table::create()
					.table(QuestionOptions::Table)
					.comment("available answers for a question object")
					.col(
						ColumnDef::new(QuestionOptions::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(QuestionOptions::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-question-options-objects")
							.from(QuestionOptions::Table, QuestionOptions::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(QuestionOptions::Name).string().not_null())
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-question-options-object").table(QuestionOptions::Table).col(QuestionOptions::Object).to_owned())
			.await?;

		manager
			.create_index(Index::create().name("index-question-options-name").table(QuestionOptions::Table).col(QuestionOptions::Name).to_owned())
			.await?;

		manager
			.create_table(
				Table::create()
					.table(QuestionAnswers::Table)
					.comment("answers received to questions")
					.col(
						ColumnDef::new(QuestionAnswers::Internal)
							.big_integer()
							.not_null()
							.primary_key()
							.auto_increment()
					)
					.col(ColumnDef::new(QuestionAnswers::Object).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-question-answers-objects")
							.from(QuestionAnswers::Table, QuestionAnswers::Object)
							.to(Objects::Table, Objects::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(QuestionAnswers::Actor).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-question-answers-actors")
							.from(QuestionAnswers::Table, QuestionAnswers::Actor)
							.to(Actors::Table, Actors::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.col(ColumnDef::new(QuestionAnswers::Answer).big_integer().not_null())
					.foreign_key(
						ForeignKey::create()
							.name("fkey-question-answers-question-options")
							.from(QuestionAnswers::Table, QuestionAnswers::Answer)
							.to(QuestionOptions::Table, QuestionOptions::Internal)
							.on_update(ForeignKeyAction::Cascade)
					)
					.to_owned()
			)
			.await?;

		manager
			.create_index(Index::create().name("index-question-answers-object").table(QuestionAnswers::Table).col(QuestionAnswers::Object).to_owned())
			.await?;


		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.add_column(ColumnDef::new(Objects::EndTime).timestamp_with_time_zone().null())
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.add_column(ColumnDef::new(Objects::IsMultipleChoicePoll).boolean().null())
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(QuestionOptions::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(QuestionAnswers::Table).to_owned())
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.drop_column(Objects::EndTime)
					.to_owned()
			)
			.await?;

		manager
			.alter_table(
				Table::alter()
					.table(Objects::Table)
					.drop_column(Objects::IsMultipleChoicePoll)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}
