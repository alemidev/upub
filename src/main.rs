pub mod server;
pub mod model;
pub mod routes;
pub mod cli;

pub mod errors;

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

	#[cfg(feature = "migrations")]
	/// apply database migrations
	Migrate,

	#[cfg(feature = "faker")]
	/// generate fake user, note and activity
	Faker{
		/// how many fake statuses to insert for root user
		count: u64,
	},

	/// fetch a single AP object
	Fetch {
		/// object id, or uri, to fetch
		uri: String,

		#[arg(long, default_value_t = false)]
		/// store fetched object in local db
		save: bool,
	},

	/// follow a remote relay
	Relay {
		/// actor url, same as with pleroma
		actor: String,

		#[arg(long, default_value_t = false)]
		/// instead of sending a follow request, send an accept
		accept: bool
	},

	/// run db maintenance tasks
	Fix {
		#[arg(long, default_value_t = false)]
		/// fix likes counts for posts
		likes: bool,

		#[arg(long, default_value_t = false)]
		/// fix shares counts for posts
		shares: bool,

		#[arg(long, default_value_t = false)]
		/// fix replies counts for posts
		replies: bool,
	},

	/// update remote users
	Update {
		#[arg(long, short, default_value_t = 7)]
		/// number of days after which users should get updated
		days: i64,
	}
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
		#[cfg(feature = "migrations")]
		CliCommand::Migrate =>
			migrations::Migrator::up(&db, None)
				.await.expect("error applying migrations"),

		#[cfg(feature = "faker")]
		CliCommand::Faker { count } =>
			cli::faker(&db, args.domain, count)
				.await.expect("error creating fake entities"),

		CliCommand::Fetch { uri, save } => 
			cli::fetch(db, args.domain, uri, save)
				.await.expect("error fetching object"),

		CliCommand::Relay { actor, accept } =>
			cli::relay(db, args.domain, actor, accept)
				.await.expect("error registering/accepting relay"),

		CliCommand::Fix { likes, shares, replies } =>
			cli::fix(db, likes, shares, replies)
				.await.expect("failed running fix task"),

		CliCommand::Update { days } =>
			cli::update_users(db, args.domain, days)
				.await.expect("error updating users"),

		CliCommand::Serve => {
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
