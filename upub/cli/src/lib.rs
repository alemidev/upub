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
	},

	/// register a new local user
	Register {
		/// username for new user, must be unique locally and cannot be changed
		username: String,

		/// password for new user
		// TODO get this with getpass rather than argv!!!!
		password: String,

		/// display name for new user
		#[arg(long = "name")]
		display_name: Option<String>,

		/// summary text for new user
		#[arg(long = "summary")]
		summary: Option<String>,

		/// url for avatar image of new user
		#[arg(long = "avatar")]
		avatar_url: Option<String>,

		/// url for banner image of new user
		#[arg(long = "banner")]
		banner_url: Option<String>,
	}
}

pub async fn run(ctx: upub::Context, command: CliCommand) -> upub::Result<()> {
	match command {
		CliCommand::Faker { count } =>
			Ok(faker(ctx, count as i64).await?),
		CliCommand::Fetch { uri, save } =>
			Ok(fetch(ctx, uri, save).await?),
		CliCommand::Relay { actor, accept } =>
			Ok(relay(ctx, actor, accept).await?),
		CliCommand::Fix { likes, shares, replies } =>
			Ok(fix(ctx, likes, shares, replies).await?),
		CliCommand::Update { days } =>
			Ok(update_users(ctx, days).await?),
		CliCommand::Register { username, password, display_name, summary, avatar_url, banner_url } =>
			Ok(register(ctx, username, password, display_name, summary, avatar_url, banner_url).await?),
	}
}
