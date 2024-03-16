pub mod model;
pub mod migrations;
pub mod activitystream;
pub mod server;

use clap::{Parser, Subcommand};
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::MigratorTrait;

#[derive(Parser)]
/// all names were taken
struct CliArgs {
	#[clap(subcommand)]
	/// command to run
	command: CliCommand,

	#[arg(short, long, default_value = "sqlite://./upub.db")]
	/// database connection uri
	database: String,

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
	Faker,
}

#[tokio::main]
async fn main() {

	let args = CliArgs::parse();

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
		.init();

	let mut opts = ConnectOptions::new(&args.database);
	opts
		.max_connections(1);

	let db = Database::connect(opts)
		.await.expect("error connecting to db");

	match args.command {
		CliCommand::Serve => server::serve(db)
			.await,

		CliCommand::Migrate => migrations::Migrator::up(&db, None)
			.await.expect("error applying migrations"),

		CliCommand::Faker => model::faker(&db)
			.await.expect("error creating fake entities"),
	}
}


