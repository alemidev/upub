use sea_orm_migration::prelude::*;

mod m20240316_000001_create_table;
mod m20240322_000001_create_relations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20240316_000001_create_table::Migration),
			Box::new(m20240322_000001_create_relations::Migration),
		]
	}
}
