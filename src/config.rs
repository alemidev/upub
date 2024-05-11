

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct Config {
	#[serde(default)]
	pub instance: InstanceConfig,

	#[serde(default)]
	pub datasource: DatasourceConfig,

	// TODO should i move app keys here?
}

#[serde_inline_default::serde_inline_default]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, serde_default::DefaultFromSerde)]
pub struct DatasourceConfig {
	#[serde_inline_default("sqlite://./upub.db".into())]
	pub connection_string: String,

	#[serde_inline_default(4)]
	pub max_connections: u32,

	#[serde_inline_default(1)]
	pub min_connections: u32,

	#[serde_inline_default(300u64)]
	pub connect_timeout_seconds: u64,

	#[serde_inline_default(300u64)]
	pub acquire_timeout_seconds: u64,

	#[serde_inline_default(1u64)]
	pub slow_query_warn_seconds: u64,

	#[serde_inline_default(true)]
	pub slow_query_warn_enable: bool,
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




impl Config {
	pub fn load(path: Option<std::path::PathBuf>) -> Self {
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
}
