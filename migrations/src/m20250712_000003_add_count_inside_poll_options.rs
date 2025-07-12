use sea_orm_migration::prelude::*;

use crate::m20250712_000001_add_question_tables::QuestionOptions;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(QuestionOptions::Table)
					.add_column(ColumnDef::new(QuestionOptions::Votes).integer().not_null().default(0))
					.to_owned()
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

		manager
			.alter_table(
				Table::alter()
					.table(QuestionOptions::Table)
					.drop_column(QuestionOptions::Votes)
					.to_owned()
			)
			.await?;

		Ok(())
	}
}

