pub mod server;
pub mod model;
pub mod routes;

pub mod tools;
pub mod errors;

#[cfg(feature = "migrations")]
mod migrations;

#[cfg(feature = "migrations")]
use sea_orm_migration::MigratorTrait;

use clap::{Parser, Subcommand};
use sea_orm::{ConnectOptions, Database, EntityTrait, IntoActiveModel};

pub use errors::UpubResult as Result;

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
		CliCommand::Migrate => migrations::Migrator::up(&db, None)
			.await.expect("error applying migrations"),

		#[cfg(feature = "faker")]
		CliCommand::Faker { count } => model::faker::faker(&db, args.domain, count)
			.await.expect("error creating fake entities"),

		CliCommand::Fetch { uri, save } => fetch(&db, &uri, save)
			.await.expect("error fetching object"),

		CliCommand::Serve => {
			let ctx = server::Context::new(db, args.domain)
				.await.expect("failed creating server context");

			use routes::activitypub::ActivityPubRouter;
			use routes::mastodon::MastodonRouter;

			let router = axum::Router::new()
				.ap_routes()
				.mastodon_routes() // no-op if mastodon feature is disabled
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


async fn fetch(db: &sea_orm::DatabaseConnection, uri: &str, save: bool) -> reqwest::Result<()> {
	use apb::{Base, Object};

	let mut node = apb::Node::from(uri);
	tracing::info!("fetching object");
	node.fetch().await?;
	tracing::info!("fetched node");

	let obj = node.get().expect("node still empty after fetch?");

	tracing::info!("fetched object:{}, name:{}", obj.id().unwrap_or(""), obj.name().unwrap_or(""));
	
	if save {
		match obj.base_type() {
			Some(apb::BaseType::Object(apb::ObjectType::Actor(_))) => {
				model::user::Entity::insert(
					model::user::Model::new(obj).unwrap().into_active_model()
				).exec(db).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Activity(_))) => {
				model::activity::Entity::insert(
					model::activity::Model::new(obj).unwrap().into_active_model()
				).exec(db).await.unwrap();
			},
			Some(apb::BaseType::Object(apb::ObjectType::Note)) => {
				model::object::Entity::insert(
					model::object::Model::new(obj).unwrap().into_active_model()
				).exec(db).await.unwrap();
			},
			Some(apb::BaseType::Object(t)) => tracing::warn!("not implemented: {:?}", t),
			Some(apb::BaseType::Link(_)) => tracing::error!("fetched another link?"),
			None => tracing::error!("no type on object"),
		}
	}

	Ok(())
}
