use std::path::PathBuf;
use clap::{Parser, Subcommand};
use sea_orm::{ConnectOptions, Database};

#[cfg(feature = "cli")]
use upub_cli as cli;

#[cfg(feature = "migrate")]
use upub_migrations as migrations;

#[cfg(feature = "serve")]
use upub_routes as routes;

#[cfg(feature = "worker")]
use upub_worker as worker;


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
	/// print current or default configuration
	Config,

	#[cfg(feature = "migrate")]
	/// apply database migrations
	Migrate,

	#[cfg(feature = "cli")]
	/// run maintenance CLI tasks
	Cli {
		#[clap(subcommand)]
		/// task to run
		command: cli::CliCommand,
	},

	#[cfg(all(feature = "serve", feature = "worker"))]
	/// start both api routes and background workers
	Monolith {
		#[arg(short, long, default_value="127.0.0.1:3000")]
		/// addr to bind and serve onto
		bind: String,

		#[arg(short, long, default_value_t = 4)]
		/// how many concurrent jobs to process with this worker
		tasks: usize,

		#[arg(short, long, default_value_t = 20)]
		/// interval for polling new tasks
		poll: u64,
	},

	#[cfg(feature = "serve")]
	/// start api routes server
	Serve {
		#[arg(short, long, default_value="127.0.0.1:3000")]
		/// addr to bind and serve onto
		bind: String,
	},

	#[cfg(feature = "worker")]
	/// start background job worker
	Work {
		/// only run tasks of this type, run all if not given
		filter: Filter,

		/// how many concurrent jobs to process with this worker
		#[arg(short, long, default_value_t = 4)]
		tasks: usize,

		#[arg(short, long, default_value_t = 20)]
		/// interval for polling new tasks
		poll: u64,
	},
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Filter {
	All,
	Local,
	Inbound,
	Outbound,
}

#[tokio::main]
async fn main() {

	let args = Args::parse();

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
		.init();

	let config = upub::Config::load(args.config);

	if matches!(args.command, Mode::Config) {
		println!("{}", toml::to_string_pretty(&config).expect("failed serializing config"));
		return;
	}

	let database = args.database.unwrap_or(config.datasource.connection_string.clone());
	let domain = args.domain.unwrap_or(config.instance.domain.clone());

	// TODO can i do connectoptions.into() or .connect() and skip these ugly bindings?
	let mut opts = ConnectOptions::new(&database);

	opts
		.sqlx_logging(true)
		.sqlx_logging_level(tracing::log::LevelFilter::Debug)
		.max_connections(config.datasource.max_connections)
		.min_connections(config.datasource.min_connections)
		.acquire_timeout(std::time::Duration::from_secs(config.datasource.acquire_timeout_seconds))
		.connect_timeout(std::time::Duration::from_secs(config.datasource.connect_timeout_seconds))
		.sqlx_slow_statements_logging_settings(
			if config.datasource.slow_query_warn_enable { tracing::log::LevelFilter::Warn } else { tracing::log::LevelFilter::Debug },
			std::time::Duration::from_secs(config.datasource.slow_query_warn_seconds)
		);

	let db = Database::connect(opts)
		.await.expect("error connecting to db");

	#[cfg(feature = "migrate")]
	if matches!(args.command, Mode::Migrate) {
		use migrations::MigratorTrait;

		migrations::Migrator::up(&db, None)
			.await
			.expect("error applying migrations");

		return;
	}

	let ctx = upub::Context::new(db, domain, config.clone())
		.await.expect("failed creating server context");

	match args.command {
		#[cfg(feature = "cli")]
		Mode::Cli { command } =>
			cli::run(ctx, command)
				.await.expect("failed running cli task"),

		#[cfg(feature = "serve")]
		Mode::Serve { bind } =>
			routes::serve(ctx, bind)
				.await.expect("failed serving api routes"),

		#[cfg(feature = "worker")]
		Mode::Work { filter, tasks, poll } =>
			worker::spawn(ctx, tasks, poll, filter.into())
				.await.expect("failed running worker"),

		#[cfg(all(feature = "serve", feature = "worker"))]
		Mode::Monolith { bind, tasks, poll } => {
			worker::spawn(ctx.clone(), tasks, poll, None);

			routes::serve(ctx, bind)
				.await.expect("failed serving api routes");
		},

		_ => unreachable!(),
	}
}

impl From<Filter> for Option<upub::model::job::JobType> {
	fn from(value: Filter) -> Self {
		match value {
			Filter::All => None,
			Filter::Local => Some(upub::model::job::JobType::Local),
			Filter::Inbound => Some(upub::model::job::JobType::Inbound),
			Filter::Outbound => Some(upub::model::job::JobType::Outbound),
		}
	}
}
