use std::sync::Arc;

use dashmap::DashMap;
use lazy_static::lazy_static;

lazy_static! {
	pub static ref CTX: Context = Context::default();
}

#[derive(Debug, Default, Clone)]
pub struct Context {
	pub cache: Arc<Cache>,
	pub timelines: Arc<Timelines>,
}

#[derive(Debug, Default)]
pub struct Cache {
	pub user: DashMap<String, serde_json::Value>,
}

#[derive(Debug, Default)]
pub struct Timelines {
}
