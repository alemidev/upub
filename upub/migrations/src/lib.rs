use sea_orm_migration::prelude::*;

mod m20240524_000001_create_actor_activity_object_tables;
mod m20240524_000002_create_relations_likes_shares;
mod m20240524_000003_create_users_auth_and_config;
mod m20240524_000004_create_addressing_deliveries;
mod m20240524_000005_create_attachments_tags_mentions;
mod m20240529_000001_add_relation_unique_index;
mod m20240605_000001_add_jobs_table;
mod m20240606_000001_add_audience_to_objects;
mod m20240607_000001_activity_ref_is_optional;

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
			Box::new(m20240529_000001_add_relation_unique_index::Migration),
			Box::new(m20240605_000001_add_jobs_table::Migration),
			Box::new(m20240606_000001_add_audience_to_objects::Migration),
			Box::new(m20240607_000001_activity_ref_is_optional::Migration),
		]
	}
}

pub use sea_orm_migration::MigratorTrait;
