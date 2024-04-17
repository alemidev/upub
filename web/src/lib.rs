mod app;
mod auth;
mod timeline;
mod view;
mod page;
mod control;

pub use app::App;
pub use timeline::Timeline;
pub use auth::{Auth, AuthToken};

pub mod prelude;

pub const URL_BASE: &str = "https://feditest.alemi.dev";
pub const URL_PREFIX: &str = "/web";
pub const NAME: &str = "μ";

use std::sync::Arc;



lazy_static::lazy_static! {
	pub static ref CACHE: ObjectCache = ObjectCache::default();
}

#[derive(Debug, Clone, Default)]
pub struct ObjectCache(pub Arc<dashmap::DashMap<String, serde_json::Value>>);

impl ObjectCache {
	pub fn get(&self, k: &str) -> Option<serde_json::Value> {
		self.0.get(k).map(|x| x.clone())
	}

	pub fn put(&self, k: String, v: serde_json::Value) {
		self.0.insert(k, v);
	}
}



pub struct Http;

impl Http {
	pub async fn request<T: serde::ser::Serialize>(
		method: reqwest::Method,
		url: &str,
		data: Option<&T>,
		token: leptos::Signal<Option<String>>
	) -> reqwest::Result<reqwest::Response> {
		use leptos::SignalGet;

		let mut req = reqwest::Client::new()
			.request(method, url);

		if let Some(auth) = token.get() {
			req = req.header("Authorization", format!("Bearer {}", auth));
		}

		if let Some(data) = data {
			req = req.json(data);
		}

		req.send()
			.await?
			.error_for_status()
	}

	pub async fn fetch<T: serde::de::DeserializeOwned>(url: &str, token: leptos::Signal<Option<String>>) -> reqwest::Result<T> {
		Self::request::<()>(reqwest::Method::GET, url, None, token)
			.await?
			.json::<T>()
			.await
	}

	pub async fn post<T: serde::ser::Serialize>(url: &str, data: &T, token: leptos::Signal<Option<String>>) -> reqwest::Result<()> {
		Self::request(reqwest::Method::POST, url, Some(data), token)
			.await?;
		Ok(())
	}
}

pub struct Uri;

impl Uri {
	pub fn full(kind: &str, id: &str) -> String {
		if id.starts_with('+') {
			id.replace('+', "https://").replace('@', "/")
		} else {
			format!("{URL_BASE}/{kind}/{id}")
		}
	}

	pub fn pretty(url: &str) -> String {
		if url.len() < 50 {
			url.replace("https://", "")
		} else {
			format!("{}..", url.replace("https://", "").get(..50).unwrap_or_default())
		}.replace('/', "\u{200B}/\u{200B}")
	}

	pub fn short(url: &str) -> String {
		if url.starts_with(URL_BASE) {
			url.split('/').last().unwrap_or_default().to_string()
		} else {
			url.replace("https://", "+").replace('/', "@")
		}
	}

	/// convert url id to valid frontend view id:
	///   /web/users/test
	///   /web/objects/+social.alemi.dev@objects@1204kasfkl
	/// accepts:
	///  - https://my.domain.net/users/root
	///  - https://other.domain.net/unexpected/path/root
	///  - +other.domain.net@users@root
	///  - root
	pub fn web(kind: &str, url: &str) -> String {
		format!("/web/{kind}/{}", Self::short(url))
	}
	
	/// convert url id to valid backend api id
	///   https://feditest.alemi.dev/users/test
	///   https://feditest.alemi.dev/users/+social.alemi.dev@users@alemi
	/// accepts:
	///  - https://my.domain.net/users/root
	///  - https://other.domain.net/unexpected/path/root
	///  - +other.domain.net@users@root
	///  - root
	pub fn api(kind: &str, url: &str) -> String {
		format!("{URL_BASE}/{kind}/{}", Self::short(url))
	}
}
