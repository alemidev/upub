

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

	// TODO should i move app keys here?
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct InstanceConfig {
	#[serde_inline_default("μpub".into())]
	pub name: String,

	#[serde_inline_default("micro social network, federated".into())]
	pub description: String,

	#[serde_inline_default("upub.social".into())]
	pub domain: String,

	#[serde(default)]
	pub contact: Option<String>,

	#[serde(default)]
	pub frontend: Option<String>,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct DatasourceConfig {
	#[serde_inline_default("sqlite://./upub.db".into())]
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
	pub slow_query_warn_seconds: u64,

	#[serde_inline_default(true)]
	pub slow_query_warn_enable: bool,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct SecurityConfig {
	#[serde(default)]
	pub allow_registration: bool,

	#[serde(default)] // TODO i don't like the name of this
	pub require_user_approval: bool,

	#[serde(default)]
	pub allow_public_debugger: bool,

	#[serde_inline_default("changeme".to_string())]
	pub proxy_secret: String,

	#[serde_inline_default(true)]
	pub show_reply_ids: bool,

	#[serde_inline_default(true)]
	pub allow_login_refresh: bool,

	#[serde_inline_default(7 * 24)]
	pub session_duration_hours: i64,

	#[serde_inline_default(2)]
	pub max_id_redirects: u32,

	#[serde_inline_default(20)]
	pub thread_crawl_depth: u32,

	#[serde_inline_default(30)]
	pub job_expiration_days: u32,

	#[serde_inline_default(100)]
	pub reinsertion_attempt_limit: u32,
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct CompatibilityConfig {
	#[serde(default)]
	pub fix_attachment_images_media_type: bool,

	#[serde(default)]
	pub add_explicit_target_to_likes_if_local: bool,
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

	pub fn frontend_url(&self, url: &str) -> Option<String> {
		Some(format!("{}{}", self.instance.frontend.as_deref()?, url))
	}
}
