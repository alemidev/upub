#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct Config {
	#[serde(default)]
	pub instance: InstanceConfig,

	#[serde(default)]
	pub datasource: DatasourceConfig,

	#[serde(default)]
	pub security: SecurityConfig,

	#[serde(default)]
	pub behavior: BehaviorConfig,

	#[serde(default)]
	pub compat: CompatibilityConfig,

	#[serde(default)]
	pub files: FileStorageConfig,

	#[serde(default)]
	pub reject: RejectConfig,

	#[serde(default)]
	pub robots: RobotsConfig,

	// TODO should i move app keys here?
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct InstanceConfig {
	#[serde_inline_default("μpub".into())]
	/// instance name, shown in noedinfo and instance actor
	pub name: String,

	#[serde_inline_default("micro social network, federated".into())]
	/// description, shown in nodeinfo and instance actor
	pub description: String,

	#[serde_inline_default("http://127.0.0.1:3000".into())]
	/// domain of current instance, must change this for prod
	pub domain: String,

	#[serde(default)]
	/// contact information for an administrator, shown in nodeinfo metadata
	pub contact: String,

	#[serde(default)]
	/// base url for frontend, will be used to compose pretty urls
	pub frontend: String,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct DatasourceConfig {
	#[serde_inline_default("sqlite://./upub.db?mode=rwc".into())]
	pub connection_string: String,

	#[serde_inline_default(32)]
	pub max_connections: u32,

	#[serde_inline_default(1)]
	pub min_connections: u32,

	#[serde_inline_default(90u64)]
	pub connect_timeout_seconds: u64,

	#[serde_inline_default(30u64)]
	pub acquire_timeout_seconds: u64,

	#[serde_inline_default(10u64)]
	/// threshold for queries to be considered slow
	pub slow_query_warn_seconds: u64,

	#[serde_inline_default(true)]
	/// enable logging warn for slow queries
	pub slow_query_warn_enable: bool,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct SecurityConfig {
	#[serde(default)]
	/// allow new users to register autonomously
	pub allow_registration: bool,

	#[serde(default)] // TODO i don't like the name of this
	/// newly registered users require manual activation
	pub require_user_approval: bool,

	#[serde(default)]
	/// allow anonymous users access to fetch debugger (explore screen)
	pub allow_public_debugger: bool,

	#[serde(default)]
	/// allow anonymous users to perform full-text searches
	pub allow_public_search: bool,

	#[serde_inline_default("definitely-change-this-in-prod".to_string())]
	/// secret for media proxy, set this to something random
	pub proxy_secret: String,

	#[serde_inline_default(true)]
	/// allow expired tokens to be refreshed
	pub allow_login_refresh: bool,

	#[serde_inline_default(7 * 24)]
	/// how long do login sessions last
	pub session_duration_hours: i64,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct BehaviorConfig {
	#[serde_inline_default(30)]
	/// max time, in seconds, before requests fail with timeout
	pub request_timeout: u64,

	#[serde_inline_default(2)]
	/// how many times we allow an object to redirect
	pub max_id_redirects: u32,

	#[serde_inline_default(20)]
	/// how deep should threads be crawled for fetching replies
	pub thread_crawl_depth: u32,

	#[serde_inline_default(30)]
	/// how long before a job is considered stale and dropped
	pub job_expiration_days: u32,

	#[serde_inline_default(100)]
	/// how many times to attempt inserting back incomplete jobs
	pub reinsertion_attempt_limit: u32,

	#[serde_inline_default(3600)]
	/// recalculate instance stats when they're older than this (in seconds)
	pub stats_max_age: i64,

	#[serde(default)]
	/// hide compatibility rules from nodeinfo metadata
	pub hide_compat_rules: bool,

	#[serde(default)]
	/// hide reject rules from nodeinfo metadata
	pub hide_reject_rules: bool,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct CompatibilityConfig {
	#[serde_inline_default(true)]
	/// compatibility with almost everything: set document type as image/video/audio according to
	/// mediaType, because almost all software sends us `Document` attachments
	pub fix_attachment_media_type: bool,

	#[serde_inline_default(true)]
	/// compatibility with mastodon and misskey (and somewhat lemmy?): notify like receiver
	pub add_explicit_target_to_likes_if_local: bool,

	#[serde_inline_default(true)]
	/// compatibility with lemmy: avoid showing images twice
	pub skip_single_attachment_if_image_is_set: bool,

	#[serde_inline_default(false)]
	/// compatibility with most relays: since they send us other server's activities, we must fetch
	/// them to verify that they aren't falsified by the relay itself. this is quite expensive, as
	/// relays send a lot of activities and we effectively end up fetching again all these, so this
	/// defaults to false
	pub verify_relayed_activities_by_fetching: bool,

	#[serde_inline_default(false)]
	/// compatibility with most relays + mastodon: since they send us other server's activities, we
	/// should fetch them to verify the relay isn't falsifying them. however, some software (cough
	/// mastodon) has fake activity ids for stuff like Undos and Updates
	/// (https://mastodon.social/users/some_user#announces/1234/undo) which makes these activities
	/// impossible to fetch and verify. by enabling this setting, activities by relays are just
	/// trusted, even if the signing actor doesn't match the activity actor. you should only enable
	/// this if you trust ALL relays sending you data
	pub trust_relayed_activities_by_registered_relays: bool,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct FileStorageConfig {
	#[serde_inline_default(false)]
	/// allow uploading files to this instance
	pub allow_uploads: bool,

	#[serde_inline_default(false)]
	/// allow downloading files from this instance
	pub allow_downloads: bool,

	#[serde_inline_default("files/".to_string())]
	/// path where media files should be stored
	pub path: String,

	#[serde(default)]
	/// url prefix to download files. this is needed if upub uploads files somewhere, but then
	/// something else handles the download itself, like nginx. you probably want to use this
	/// together with allow_downloads:false. don't include trailing slash
	pub download_base: Option<String>,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct RejectConfig {
	#[serde(default)]
	/// instances for which all below rules apply
	pub everything: Vec<String>,

	#[serde(default)]
	/// discard incoming activities from these instances
	pub incoming: Vec<String>,

	#[serde(default)]
	/// prevent fetching content from these instances
	pub fetch: Vec<String>,

	#[serde(default)]
	/// prevent content from these instances from being displayed publicly
	/// this effectively removes the public (aka NULL) addressing: only other addressees (followers,
	/// mentions) will be able to see content from these instances on timelines and directly
	pub public: Vec<String>,

	#[serde(default)]
	/// prevent proxying media coming from these instances
	pub media: Vec<String>,

	#[serde(default)]
	/// skip delivering to these instances
	pub delivery: Vec<String>,

	#[serde(default)]
	/// prevent fetching private content from these instances. effectively downgrades all requests
	/// from these instance as anonymous requests.
	pub access: Vec<String>,

	#[serde(default)]
	/// reject any request from these instances (ineffective as they can still fetch anonymously)
	pub requests: Vec<String>,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct RobotsConfig {
	#[serde(default)]
	/// user-agents allowed to access this instance
	pub allow: Vec<String>,

	#[serde_inline_default(vec!["*".to_string()])]
	/// user-agents disallowed from accessing this instance
	pub disallow: Vec<String>,
}

impl Config {
	pub fn load(path: Option<&std::path::PathBuf>) -> Self {
		let Some(cfg_path) = path else { return Config::default() };
		match std::fs::read_to_string(cfg_path) {
			Ok(x) => match toml::from_str(&x) {
				Ok(cfg) => return cfg,
				Err(e) => tracing::error!("failed parsing config file: {e}"),
			},
			Err(e) => tracing::error!("failed reading config file: {e}"),
		}
		Config::default()
	}

	// TODO this is very magic... can we do better? maybe formalize frontend url as an attribute of
	//      our application?
	pub fn frontend_url(&self, url: &str) -> Option<String> {
		if !self.instance.frontend.is_empty() {
			Some(format!("{}{url}", self.instance.frontend))
		} else {
			None
		}
	}
}
