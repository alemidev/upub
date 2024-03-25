pub mod activitystream;
pub mod activitypub;

mod model;
mod migrations;
mod server;
mod router;
mod errors;
mod auth;
mod dispatcher;
mod fetcher;

use clap::{Parser, Subcommand};
use sea_orm::{ConnectOptions, Database, EntityTrait, IntoActiveModel};
use sea_orm_migration::MigratorTrait;

use crate::activitystream::{BaseType, ObjectType};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
/// all names were taken
struct CliArgs {
	#[clap(subcommand)]
	/// command to run
	command: CliCommand,

	#[arg(short = 'd', long = "db", default_value = "sqlite://./upub.db")]
	/// database connection uri
	database: String,

	#[arg(short = 'D', long, default_value = "http://localhost:3000")]
	/// instance base domain, for AP ids
	domain: String,

	#[arg(long, default_value_t=false)]
	/// run with debug level tracing
	debug: bool,
}

#[derive(Clone, Subcommand)]
enum CliCommand {
	/// run fediverse server
	Serve ,

	/// apply database migrations
	Migrate,

	/// generate fake user, note and activity
	Faker{
		/// how many fake statuses to insert for root user
		count: usize,
	},

	/// fetch a single AP object
	Fetch {
		/// object id, or uri, to fetch
		uri: String,

		#[arg(long, default_value_t = false)]
		/// store fetched object in local db
		save: bool,
	},
}

#[tokio::main]
async fn main() {

	let args = CliArgs::parse();

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
		.init();

	// TODO can i do connectoptions.into() or .connect() and skip these ugly bindings?
	let mut opts = ConnectOptions::new(&args.database);

	opts
		.sqlx_logging_level(tracing::log::LevelFilter::Debug);

	let db = Database::connect(opts)
		.await.expect("error connecting to db");

	match args.command {
		CliCommand::Serve => router::serve(db, args.domain)
			.await,

		CliCommand::Migrate => migrations::Migrator::up(&db, None)
			.await.expect("error applying migrations"),

		CliCommand::Faker => model::faker::faker(&db, args.domain)
			.await.expect("error creating fake entities"),

		CliCommand::Fetch { uri, save } => fetch(&db, &uri, save)
			.await.expect("error fetching object"),
	}
}



async fn fetch(db: &sea_orm::DatabaseConnection, uri: &str, save: bool) -> reqwest::Result<()> {
	use crate::activitystream::{Base, Object};

	let mut node = activitystream::Node::from(uri);
	tracing::info!("fetching object");
	node.fetch().await?;
	tracing::info!("fetched node");

	let obj = node.get().expect("node still empty after fetch?");

	tracing::info!("fetched object:{}, name:{}", obj.id().unwrap_or(""), obj.name().unwrap_or(""));
	
	if save {
		match obj.base_type() {
			Some(BaseType::Object(ObjectType::Actor(_))) => {
				model::user::Entity::insert(
					model::user::Model::new(&obj).unwrap().into_active_model()
				).exec(db).await.unwrap();
			},
			Some(BaseType::Object(ObjectType::Activity(_))) => {
				model::activity::Entity::insert(
					model::activity::Model::new(&obj).unwrap().into_active_model()
				).exec(db).await.unwrap();
			},
			Some(BaseType::Object(ObjectType::Note)) => {
				model::object::Entity::insert(
					model::object::Model::new(&obj).unwrap().into_active_model()
				).exec(db).await.unwrap();
			},
			Some(BaseType::Object(t)) => tracing::warn!("not implemented: {:?}", t),
			Some(BaseType::Link(_)) => tracing::error!("fetched another link?"),
			None => tracing::error!("no type on object"),
		}
	}

	Ok(())
}
