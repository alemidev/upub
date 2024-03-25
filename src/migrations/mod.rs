use sea_orm_migration::prelude::*;

mod m20240316_000001_create_table;
mod m20240322_000001_create_relations;
mod m20240322_000002_add_likes_shares;
mod m20240322_000003_add_indexes;
mod m20240323_000001_add_user_configs;
mod m20240323_000002_add_simple_credentials;
mod m20240324_000001_add_addressing;
mod m20240325_000001_add_deliveries;
mod m20240325_000002_add_system_key;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20240316_000001_create_table::Migration),
			Box::new(m20240322_000001_create_relations::Migration),
			Box::new(m20240322_000002_add_likes_shares::Migration),
			Box::new(m20240322_000003_add_indexes::Migration),
			Box::new(m20240323_000001_add_user_configs::Migration),
			Box::new(m20240323_000002_add_simple_credentials::Migration),
			Box::new(m20240324_000001_add_addressing::Migration),
			Box::new(m20240325_000001_add_deliveries::Migration),
			Box::new(m20240325_000002_add_system_key::Migration),
		]
	}
}
