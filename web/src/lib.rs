mod app;
mod auth;
mod components;
mod page;
mod control;

pub use app::App;

pub mod prelude;

pub const URL_BASE: &str = "https://feditest.alemi.dev";
pub const URL_PREFIX: &str = "/web";
pub const URL_SENSITIVE: &str = "https://cdn.alemi.dev/social/nsfw.png";
pub const NAME: &str = "μ";

use std::sync::Arc;
use auth::Auth;



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

	pub async fn fetch(&self, k: &str, kind: FetchKind) -> reqwest::Result<Object> {
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


#[derive(Debug, Clone)]
pub enum FetchKind {
	User,
	Object,
	Activity,
	Context,
}

impl AsRef<str> for FetchKind {
	fn as_ref(&self) -> &str {
		match self {
			Self::User => "users",
			Self::Object => "objects",
			Self::Activity => "activities",
			Self::Context => "context",
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
		use leptos::SignalGet;

		let mut req = reqwest::Client::new()
			.request(method, url);

		if let Some(auth) = auth.token.get().filter(|x| !x.is_empty()) {
			req = req.header("Authorization", format!("Bearer {}", auth));
		}

		if let Some(data) = data {
			req = req.json(data);
		}

		req.send()
			.await?
			.error_for_status()
	}

	pub async fn fetch<T: serde::de::DeserializeOwned>(url: &str, token: Auth) -> reqwest::Result<T> {
		Self::request::<()>(reqwest::Method::GET, url, None, token)
			.await?
			.json::<T>()
			.await
	}

	pub async fn post<T: serde::ser::Serialize>(url: &str, data: &T, token: Auth) -> reqwest::Result<()> {
		Self::request(reqwest::Method::POST, url, Some(data), token)
			.await?;
		Ok(())
	}
}

pub struct Uri;

impl Uri {
	pub fn full(kind: FetchKind, id: &str) -> String {
		let kind = kind.as_ref();
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
	pub fn web(kind: FetchKind, url: &str) -> String {
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
	pub fn api(kind: FetchKind, url: &str, fetch: bool) -> String {
		let kind = kind.as_ref();
		format!("{URL_BASE}/{kind}/{}{}", Self::short(url), if fetch { "?fetch=true" } else { "" })
	}
}
