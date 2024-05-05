pub mod server; // TODO there are some methods that i dont use yet, make it public so that ra shuts up
mod model;
mod routes;

mod errors;

mod config;

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "migrations")]
mod migrations;

#[cfg(feature = "migrations")]
use sea_orm_migration::MigratorTrait;

use clap::{Parser, Subcommand};
use sea_orm::{ConnectOptions, Database};

pub use errors::UpubResult as Result;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
/// all names were taken
struct Args {
	#[clap(subcommand)]
	/// command to run
	command: Mode,

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
enum Mode {
	/// run fediverse server
	Serve ,

	#[cfg(feature = "migrations")]
	/// apply database migrations
	Migrate,

	#[cfg(feature = "cli")]
	/// run maintenance CLI tasks
	Cli {
		#[clap(subcommand)]
		/// task to run
		command: cli::CliCommand,
	},
}

#[tokio::main]
async fn main() {

	let args = Args::parse();

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
		#[cfg(feature = "migrations")]
		Mode::Migrate =>
			migrations::Migrator::up(&db, None)
				.await.expect("error applying migrations"),

		#[cfg(feature = "cli")]
		Mode::Cli { command } =>
			cli::run(command, db, args.domain)
				.await.expect("failed running cli task"),

		Mode::Serve => {
			let ctx = server::Context::new(db, args.domain)
				.await.expect("failed creating server context");

			use routes::activitypub::ActivityPubRouter;
			use routes::mastodon::MastodonRouter;

			let router = axum::Router::new()
				.ap_routes()
				.mastodon_routes() // no-op if mastodon feature is disabled
				.layer(CorsLayer::permissive())
				.layer(TraceLayer::new_for_http())
				.with_state(ctx);

			// run our app with hyper, listening locally on port 3000
			let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
				.await.expect("could not bind tcp socket");

			axum::serve(listener, router)
				.await
				.expect("failed serving application")
		},
	}
}
