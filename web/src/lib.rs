#![recursion_limit = "256"] // oh nooo leptos...

mod auth;
mod app;
mod components;
mod page;
mod config;
mod groups;

mod actors;
mod activities;
mod objects;
mod timeline;

use apb::{Activity, Object, Actor, Base};
pub use app::App;
pub use config::Config;
pub use auth::Auth;

pub mod prelude;

pub const URL_BASE: &str = "https://dev.upub.social";
pub const URL_PREFIX: &str = "/web";
pub const URL_SENSITIVE: &str = "https://cdn.alemi.dev/social/nsfw.png";
pub const FALLBACK_IMAGE_URL: &str = "https://cdn.alemi.dev/social/gradient.png";
pub const NAME: &str = "μ";
pub const DEFAULT_COLOR: &str = "#BF616A";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::{ops::Deref, sync::Arc};
use uriproxy::UriClass;

pub type Doc = Arc<serde_json::Value>;

pub mod cache {
	use super::DashmapCache;
	lazy_static::lazy_static! {
		pub static ref OBJECTS: DashmapCache<super::Doc> = DashmapCache::default();
		pub static ref WEBFINGER: DashmapCache<String> = DashmapCache::default();
		pub static ref TIMELINES: DashmapCache<(Option<String>, Vec<String>)> = DashmapCache::default();
	}
}

#[derive(Debug)]
pub enum LookupStatus<T> {
	Resolving, // TODO use this to avoid fetching twice!
	Found(T),
	NotFound,
}

impl<T> LookupStatus<T> {
	fn inner(&self) -> Option<&T> {
		if let Self::Found(x) = self {
			return Some(x);
		}
		None
	}
}

pub trait Cache {
	type Item;

	fn lookup(&self, key: &str) -> Option<impl Deref<Target = LookupStatus<Self::Item>>>;
	fn store(&self, key: &str, value: Self::Item) -> Option<Self::Item>;
	fn invalidate(&self, key: &str);
	fn clear(&self);

	fn get(&self, key: &str) -> Option<Self::Item> where Self::Item : Clone {
		Some(self.lookup(key)?.deref().inner()?.clone())
	}

	fn get_or(&self, key: &str, or: Self::Item) -> Self::Item where Self::Item : Clone {
		self.get(key).unwrap_or(or)
	}

	fn get_or_default(&self, key: &str) -> Self::Item where Self::Item : Clone + Default {
		self.get(key).unwrap_or_default()
	}
}

#[derive(Default, Clone)]
pub struct DashmapCache<T>(Arc<dashmap::DashMap<String, LookupStatus<T>>>);

impl<T> Cache for DashmapCache<T> {
	type Item = T;

	fn lookup(&self, key: &str) -> Option<impl Deref<Target = LookupStatus<Self::Item>>> {
		self.0.get(key)
	}

	fn store(&self, key: &str, value: Self::Item) -> Option<Self::Item> {
		self.0.insert(key.to_string(), LookupStatus::Found(value))
			.and_then(|x| if let LookupStatus::Found(x) = x { Some(x) } else { None } )
	}

	fn invalidate(&self, key: &str) {
		self.0.remove(key);
	}

	fn clear(&self) {
		self.0.clear();
	}
}

impl DashmapCache<Doc> {
	pub async fn fetch(&self, key: &str, kind: UriClass, auth: Auth) -> Option<Doc> {
		let full_key = Uri::full(kind, key);
		tracing::debug!("resolving {key} -> {full_key}");
		match self.get(&full_key) {
			Some(x) => Some(x),
			None => {
				match Http::fetch::<serde_json::Value>(&Uri::api(kind, key, true), auth).await {
					Ok(obj) => {
						let obj = Arc::new(obj);
						self.include(obj.clone());
						Some(obj)
					},
					Err(e) => {
						tracing::error!("failed loading object from backend: {e}");
						None
					},
				}
			},
		}
	}

	pub fn include(&self, obj: Doc) {
		if let Ok(id) = obj.id() {
			tracing::debug!("storing object {id}: {obj}");
			cache::OBJECTS.store(&id, obj.clone());
			if obj.actor_type().is_ok() {
				if let Ok(url) = obj.url().id() {
					cache::WEBFINGER.store(&url, id);
				}
			}
		}
		if let Ok(sub_obj) = obj.object().into_inner() {
			if let Ok(sub_id) = sub_obj.id() {
				tracing::debug!("storing sub object {sub_id}: {sub_obj}");
				cache::OBJECTS.store(&sub_id, Arc::new(sub_obj));
			}
		}
	}

	pub async fn preload(&self, key: String, kind: UriClass, auth: Auth) -> Option<Doc> {
		let doc = self.fetch(&key, kind, auth).await?;
		let mut sub_tasks = Vec::new();

		match kind {
			UriClass::Activity => {
				if let Ok(actor) = doc.actor().id() {
					sub_tasks.push(self.preload(actor, UriClass::Actor, auth));
				}
				let clazz = match doc.activity_type().unwrap_or(apb::ActivityType::Activity) {
					// TODO activities like Announce or Update may be multiple things, we can't know before
					apb::ActivityType::Accept(_) => UriClass::Activity,
					apb::ActivityType::Reject(_) => UriClass::Activity,
					apb::ActivityType::Undo => UriClass::Activity,
					apb::ActivityType::Follow => UriClass::Actor,
					_ => UriClass::Object,
				};
				if let Ok(object) = doc.object().id() {
					sub_tasks.push(self.preload(object, clazz, auth));
				}
			},
			UriClass::Object => {
				if let Ok(actor) = doc.attributed_to().id() {
					sub_tasks.push(self.preload(actor, UriClass::Actor, auth));
				}
				if let Ok(quote) = doc.quote_url().id() {
					sub_tasks.push(self.preload(quote, UriClass::Object, auth));
				}
			},
			_ => {},
		}

		futures::future::join_all(sub_tasks).await;

		Some(doc)
	}
}

impl DashmapCache<String> {
	pub async fn blocking_resolve(&self, user: &str, domain: &str) -> Option<String> {
		if let Some(x) = self.resource(user, domain) { return Some(x); }
		self.fetch(user, domain).await;
		self.resource(user, domain)
	}

	pub fn resolve(&self, user: &str, domain: &str) -> Option<String> {
		if let Some(x) = self.resource(user, domain) { return Some(x); }
		let (_self, user, domain) = (self.clone(), user.to_string(), domain.to_string());
		leptos::task::spawn_local(async move { _self.fetch(&user, &domain).await });
		None
	}

	fn resource(&self, user: &str, domain: &str) -> Option<String> {
		let query = format!("{user}@{domain}");
		self.get(&query)
	}

	async fn fetch(&self, user: &str, domain: &str) {
		let query = format!("{user}@{domain}");
		self.0.insert(query.to_string(), LookupStatus::Resolving);
		match reqwest::get(format!("{URL_BASE}/.well-known/webfinger?resource=acct:{query}")).await {
			Ok(res) => match res.error_for_status() {
				Ok(res) => match res.json::<jrd::JsonResourceDescriptor>().await {
					Ok(doc) => {
						if let Some(uid) = doc.links.into_iter().find(|x| x.rel == "self").and_then(|x| x.href) {
							self.0.insert(query, LookupStatus::Found(uid));
						} else {
							self.0.insert(query, LookupStatus::NotFound);
						}
					},
					Err(e) => {
						tracing::error!("invalid webfinger response: {e:?}");
						self.0.remove(&query);
					},
				},
				Err(e) => {
					tracing::error!("could not resolve webfinbger: {e:?}");
					self.0.insert(query, LookupStatus::NotFound);
				},
			},
			Err(e) => {
				tracing::error!("failed accessing webfinger server: {e:?}");
				self.0.remove(&query);
			},
		}
	}
}

use leptos_router::params::Params; // TODO can i remove this?
#[derive(Clone, leptos::Params, PartialEq)]
pub struct IdParam {
	id: Option<String>,
}

pub struct Http;

impl Http {
	pub async fn request<T: serde::ser::Serialize>(
		method: reqwest::Method,
		url: &str,
		data: Option<&T>,
		auth: Auth,
	) -> reqwest::Result<reqwest::Response> {
		use leptos::prelude::GetUntracked;

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

	pub fn pretty(url: &str, len: usize) -> String {
		let bare = url.replace("https://", "");
		if bare.len() < len {
			bare
		} else {
			format!("{}..", bare.get(..len).unwrap_or_default())
		}
			//.replace('/', "\u{200B}/\u{200B}")
	}

	pub fn short(url: &str) -> String {
		if url.starts_with(URL_BASE) || url.starts_with('/') {
			uriproxy::decompose(url)
		} else if url.starts_with("https://") || url.starts_with("http://") {
			uriproxy::compact(url)
		} else {
			url.to_string()
		}
	}

	/// convert url id to valid frontend view id:
	///
	/// accepts:
	///
	pub fn web(kind: UriClass, url: &str) -> String {
		let kind = kind.as_ref();
		format!("/web/{kind}/{}", Self::short(url))
	}
	
	/// convert url id to valid backend api id
	///
	/// accepts:
	///
	pub fn api(kind: UriClass, url: &str, fetch: bool) -> String {
		let kind = kind.as_ref();
		format!("{URL_BASE}/{kind}/{}{}", Self::short(url), if fetch { "?fetch=true" } else { "" })
	}

	pub fn domain(full: &str) -> String {
		full
			.replacen("https://", "", 1)
			.replacen("http://", "", 1)
			.split('/')
			.next()
			.unwrap_or_default()
			.to_string()
	}
}
