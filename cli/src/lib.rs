mod count;
pub use count::*;

mod fix_activities;
pub use fix_activities::*;

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

mod nuke;
pub use nuke::*;

mod thread;
pub use thread::*;

mod cloak;
pub use cloak::*;

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

		#[arg(long)]
		/// use this actor's private key to fetch
		fetch_as: Option<String>,
	},

	/// act on remote relay actors at instance level
	Relay {
		#[clap(subcommand)]
		/// action to take against this relay
		action: RelayCommand,
	},

	/// recount object statistics
	Count {
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

	/// update remote actors
	Update {
		#[arg(long, short, default_value_t = 10)]
		/// number of days after which actors should get updated
		days: i64,

		#[arg(long)]
		/// stop after updating this many actors
		limit: Option<u64>,
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
	},

	/// break all user relations so that instance can be shut down
	Nuke {
		/// unless this is set, nuke will be a dry run
		#[arg(long, default_value_t = false)]
		for_real: bool,

		/// also send Delete activities for all local objects
		#[arg(long, default_value_t = false)]
		delete_objects: bool,
	},

	/// attempt to fix broken threads and completely gather their context
	Thread {
		
	},

	/// replaces all attachment urls with proxied local versions (only useful for old instances)
	Cloak {
		/// also cloak objects image urls
		#[arg(long, default_value_t = false)]
		objects: bool,

		/// also cloak actor images
		#[arg(long, default_value_t = false)]
		actors: bool,

		/// also replace urls inside post contents
		#[arg(long, default_value_t = false)]
		contents: bool,

		/// also re-cloak already cloaked urls, useful if changing cloak secret
		#[arg(long, default_value_t = false)]
		re_cloak: bool,
	},

	/// restore activities links, only needed for very old installs
	FixActivities {
		/// restore like activity links
		#[arg(long, default_value_t = false)]
		likes: bool,

		/// restore announces activity links
		#[arg(long, default_value_t = false)]
		announces: bool,
	},
}

pub async fn run(ctx: upub::Context, command: CliCommand) -> Result<(), Box<dyn std::error::Error>> {
	tracing::info!("running cli task: {command:?}");
	match command {
		CliCommand::Faker { count } =>
			Ok(faker(ctx, count as i64).await?),
		CliCommand::Fetch { uri, save, fetch_as } =>
			Ok(fetch(ctx, uri, save, fetch_as).await?),
		CliCommand::Relay { action } =>
			Ok(relay(ctx, action).await?),
		CliCommand::Count { likes, shares, replies } =>
			Ok(count(ctx, likes, shares, replies).await?),
		CliCommand::Update { days, limit } =>
			Ok(update_users(ctx, days, limit).await?),
		CliCommand::Register { username, password, display_name, summary, avatar_url, banner_url } =>
			Ok(register(ctx, username, password, display_name, summary, avatar_url, banner_url).await?),
		CliCommand::Nuke { for_real, delete_objects } =>
			Ok(nuke(ctx, for_real, delete_objects).await?),
		CliCommand::Thread { } =>
			Ok(thread(ctx).await?),
		CliCommand::Cloak { objects, actors, contents, re_cloak } =>
			Ok(cloak(ctx, contents, objects, actors, re_cloak).await?),
		CliCommand::FixActivities { likes, announces } =>
			Ok(fix_activities(ctx, likes, announces).await?),
	}
}
