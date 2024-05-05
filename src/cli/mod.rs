mod fix;
pub use fix::*;

mod fetch;
pub use fetch::*;

mod faker;
pub use faker::*;

mod relay;
pub use relay::*;

mod register;
pub use register::*;

mod update;
pub use update::*;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum CliCommand {
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

pub async fn run(
	command: CliCommand,
	db: sea_orm::DatabaseConnection,
	domain: String,
) -> crate::Result<()> {
	match command {
		CliCommand::Faker { count } =>
			Ok(faker(&db, domain, count).await?),
		CliCommand::Fetch { uri, save } =>
			Ok(fetch(db, domain, uri, save).await?),
		CliCommand::Relay { actor, accept } =>
			Ok(relay(db, domain, actor, accept).await?),
		CliCommand::Fix { likes, shares, replies } =>
			Ok(fix(db, likes, shares, replies).await?),
		CliCommand::Update { days } =>
			Ok(update_users(db, domain, days).await?),
	}
}
