mod auth;
mod app;
mod components;
mod page;
mod config;

pub use app::App;
pub use config::Config;
pub use auth::Auth;

pub mod prelude;

pub const URL_BASE: &str = "https://feditest.alemi.dev";
pub const URL_PREFIX: &str = "/web";
pub const URL_SENSITIVE: &str = "https://cdn.alemi.dev/social/nsfw.png";
pub const DEFAULT_AVATAR_URL: &str = "https://cdn.alemi.dev/social/gradient.png";
pub const NAME: &str = "μ";
pub const DEFAULT_COLOR: &str = "#BF616A";

use std::sync::Arc;
use uriproxy::UriClass;



lazy_static::lazy_static! {
	pub static ref CACHE: ObjectCache = ObjectCache::default();
}

pub type Object = Arc<serde_json::Value>;

#[derive(Debug, Clone, Default)]
pub struct ObjectCache(pub Arc<dashmap::DashMap<String, Object>>);

impl ObjectCache {
	pub fn get(&self, k: &str) -> Option<Object> {
		self.0.get(k).map(|x| x.clone())
	}

	pub fn get_or(&self, k: &str, or: Object) -> Object {
		self.get(k).unwrap_or(or)
	}

	pub fn put(&self, k: String, v: Object) {
		self.0.insert(k, v);
	}

	pub async fn fetch(&self, k: &str, kind: UriClass) -> reqwest::Result<Object> {
		match self.get(k) {
			Some(x) => Ok(x),
			None => {
				let obj = reqwest::get(Uri::api(kind, k, true))
					.await?
					.json::<serde_json::Value>()
					.await?;
				self.put(k.to_string(), Arc::new(obj));
				Ok(self.get(k).expect("not found in cache after insertion"))
			}
		}
	}
}


pub struct Http;

impl Http {
	pub async fn request<T: serde::ser::Serialize>(
		method: reqwest::Method,
		url: &str,
		data: Option<&T>,
		auth: Auth,
	) -> reqwest::Result<reqwest::Response> {
		use leptos::SignalGetUntracked;

		let mut req = reqwest::Client::new()
			.request(method, url);

		if let Some(auth) = auth.token.get_untracked().filter(|x| !x.is_empty()) {
			req = req.header("Authorization", format!("Bearer {}", auth));
		}

		if let Some(data) = data {
			req = req.json(data);
		}

		req.send().await
	}

	pub async fn fetch<T: serde::de::DeserializeOwned>(url: &str, token: Auth) -> reqwest::Result<T> {
		Self::request::<()>(reqwest::Method::GET, url, None, token)
			.await?
			.error_for_status()?
			.json::<T>()
			.await
	}

	pub async fn post<T: serde::ser::Serialize>(url: &str, data: &T, token: Auth) -> reqwest::Result<()> {
		Self::request(reqwest::Method::POST, url, Some(data), token)
			.await?
			.error_for_status()?;
		Ok(())
	}
}

pub struct Uri;

impl Uri {
	pub fn full(kind: UriClass, id: &str) -> String {
		uriproxy::uri(URL_BASE, kind, id)
	}

	pub fn pretty(url: &str) -> String {
		let bare = url.replace("https://", "");
		if url.len() < 50 {
			bare
		} else {
			format!("{}..", bare.get(..50).unwrap_or_default())
		}.replace('/', "\u{200B}/\u{200B}")
	}

	pub fn short(url: &str) -> String {
		if url.starts_with(URL_BASE) || url.starts_with('/') {
			uriproxy::decompose_id(url)
		} else if url.starts_with("https://") || url.starts_with("http") {
			uriproxy::compact_id(url)
		} else {
			url.to_string()
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
	pub fn web(kind: UriClass, url: &str) -> String {
		let kind = kind.as_ref();
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
	pub fn api(kind: UriClass, url: &str, fetch: bool) -> String {
		let kind = kind.as_ref();
		format!("{URL_BASE}/{kind}/{}{}", Self::short(url), if fetch { "?fetch=true" } else { "" })
	}
}
