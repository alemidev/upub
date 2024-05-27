mod server;
mod model;
mod routes;

pub mod errors;
mod config;

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "migrations")]
mod migrations;

#[cfg(feature = "migrations")]
use sea_orm_migration::MigratorTrait;
use std::path::PathBuf;
use config::Config;

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

	/// path to config file, leave empty to not use any
	#[arg(short, long)]
	config: Option<PathBuf>,

	#[arg(long = "db")]
	/// database connection uri, overrides config value
	database: Option<String>,

	#[arg(long)]
	/// instance base domain, for AP ids, overrides config value
	domain: Option<String>,

	#[arg(long, default_value_t=false)]
	/// run with debug level tracing
	debug: bool,
}

#[derive(Clone, Subcommand)]
enum Mode {
	/// run fediverse server
	Serve {
		#[arg(short, long, default_value="127.0.0.1:3000")]
		/// addr to bind and serve onto
		bind: String,
	},

	/// print current or default configuration
	Config,

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

	let config = Config::load(args.config);

	let database = args.database.unwrap_or(config.datasource.connection_string.clone());
	let domain = args.domain.unwrap_or(config.instance.domain.clone());

	// TODO can i do connectoptions.into() or .connect() and skip these ugly bindings?
	let mut opts = ConnectOptions::new(&database);

	opts
		.sqlx_logging_level(tracing::log::LevelFilter::Debug)
		.max_connections(config.datasource.max_connections)
		.min_connections(config.datasource.min_connections)
		.acquire_timeout(std::time::Duration::from_secs(config.datasource.acquire_timeout_seconds))
		.connect_timeout(std::time::Duration::from_secs(config.datasource.connect_timeout_seconds))
		.sqlx_slow_statements_logging_settings(
			if config.datasource.slow_query_warn_enable { tracing::log::LevelFilter::Warn } else { tracing::log::LevelFilter::Off },
			std::time::Duration::from_secs(config.datasource.slow_query_warn_seconds)
		);

	let db = Database::connect(opts)
		.await.expect("error connecting to db");

	match args.command {
		#[cfg(feature = "migrations")]
		Mode::Migrate =>
			migrations::Migrator::up(&db, None)
				.await.expect("error applying migrations"),

		#[cfg(feature = "cli")]
		Mode::Cli { command } =>
			cli::run(command, db, domain, config)
				.await.expect("failed running cli task"),

		Mode::Config => println!("{}", toml::to_string_pretty(&config).expect("failed serializing config")),

		Mode::Serve { bind } => {
			let ctx = server::Context::new(db, domain, config)
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
			let listener = tokio::net::TcpListener::bind(bind)
				.await.expect("could not bind tcp socket");

			axum::serve(listener, router)
				.await
				.expect("failed serving application")
		},
	}
}
