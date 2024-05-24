use sea_orm_migration::prelude::*;

mod m20240524_000001_create_actor_activity_object_tables;
mod m20240524_000002_create_relations_likes_shares;
mod m20240524_000003_create_users_auth_and_config;
mod m20240524_000004_create_addressing_deliveries;
mod m20240524_000005_create_attachments_tags_mentions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20240524_000001_create_actor_activity_object_tables::Migration),
			Box::new(m20240524_000002_create_relations_likes_shares::Migration),
			Box::new(m20240524_000003_create_users_auth_and_config::Migration),
			Box::new(m20240524_000004_create_addressing_deliveries::Migration),
			Box::new(m20240524_000005_create_attachments_tags_mentions::Migration),
		]
	}
}
