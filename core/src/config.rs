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
	pub compat: CompatibilityConfig,

	#[serde(default)]
	pub files: FileStorageConfig,

	#[serde(default)]
	pub reject: RejectConfig,

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
	/// contact information for an administrator, currently unused
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

	#[serde_inline_default(30)]
	/// max time, in seconds, before requests fail with timeout
	pub request_timeout: u64,

	#[serde_inline_default("definitely-change-this-in-prod".to_string())]
	/// secret for media proxy, set this to something random
	pub proxy_secret: String,

	#[serde_inline_default(true)]
	/// allow expired tokens to be refreshed
	pub allow_login_refresh: bool,

	#[serde_inline_default(7 * 24)]
	/// how long do login sessions last
	pub session_duration_hours: i64,

	#[serde_inline_default(2)]
	/// how many times we allow an object to redirect
	pub max_id_redirects: u32, // TODO not sure it fits here

	#[serde_inline_default(20)]
	/// how deep should threads be crawled for fetching replies
	pub thread_crawl_depth: u32, // TODO doesn't really fit here

	#[serde_inline_default(30)]
	/// how long before a job is considered stale and dropped
	pub job_expiration_days: u32, // TODO doesn't really fit here

	#[serde_inline_default(100)]
	/// how many times to attempt inserting back incomplete jobs
	pub reinsertion_attempt_limit: u32, // TODO doesn't really fit here
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct CompatibilityConfig {
	#[serde_inline_default(true)]
	/// compatibility with almost everything: set image attachments as images
	pub fix_attachment_images_media_type: bool,

	#[serde_inline_default(true)]
	/// compatibility with mastodon and misskey (and somewhat lemmy?): notify like receiver
	pub add_explicit_target_to_likes_if_local: bool,

	#[serde_inline_default(true)]
	/// compatibility with lemmy: avoid showing images twice
	pub skip_single_attachment_if_image_is_set: bool,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct FileStorageConfig {
	#[serde_inline_default("files/".to_string())]
	/// path where media files should be stored
	pub path: String,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct RejectConfig {
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
	/// prevent fetching private content from these instances
	pub access: Vec<String>,

	#[serde(default)]
	/// reject any request from these instances (ineffective as they can still fetch anonymously)
	pub requests: Vec<String>,
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
