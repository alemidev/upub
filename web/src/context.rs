use std::{collections::BTreeSet, sync::Arc};

use apb::{Activity, ActivityMut, Base, Collection, CollectionPage};
use dashmap::DashMap;
use leptos::{create_rw_signal, create_signal, leptos_dom::logging::console_warn, ReadSignal, RwSignal, Signal, SignalGet, SignalSet, WriteSignal};

use crate::URL_BASE;

lazy_static::lazy_static! {
	pub static ref CACHE: ObjectCache = ObjectCache::default();
}

#[derive(Debug, Clone, Default)]
pub struct ObjectCache(pub Arc<DashMap<String, serde_json::Value>>);

impl ObjectCache {
	pub fn get(&self, k: &str) -> Option<serde_json::Value> {
		self.0.get(k).map(|x| x.clone())
	}

	pub fn put(&self, k: String, v: serde_json::Value) {
		self.0.insert(k, v);
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

// impl ObjectCache {
// 	pub async fn user(&self, id: &str, token: Option<&str>) -> Option<serde_json::Value> {
// 		match self.actors.get(id) {
// 			Some(x) => Some(x.clone()),
// 			None => {
// 				let mut req = reqwest::Client::new()
// 					.get(format!("{URL_BASE}/users/+?id={id}"));
// 				if let Some(token) = token {
// 					req = req.header("Authorization", format!("Bearer {token}"));
// 				}
// 				let user = req
// 					.send()
// 					.await.ok()?
// 					.json::<serde_json::Value>()
// 					.await.ok()?;
// 
// 				self.actors.insert(id.to_string(), user.clone());
// 
// 				Some(user)
// 			},
// 		}
// 	}
// }

pub struct Http;

impl Http {
	pub async fn request<T: serde::ser::Serialize>(
		method: reqwest::Method,
		url: &str,
		data: Option<&T>,
		token: Signal<Option<String>>
	) -> reqwest::Result<reqwest::Response> {
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

	pub async fn fetch<T: serde::de::DeserializeOwned>(url: &str, token: Signal<Option<String>>) -> reqwest::Result<T> {
		Self::request::<()>(reqwest::Method::GET, url, None, token)
			.await?
			.json::<T>()
			.await
	}

	pub async fn post<T: serde::ser::Serialize>(url: &str, data: &T, token: Signal<Option<String>>) -> reqwest::Result<()> {
		Self::request(reqwest::Method::POST, url, Some(data), token)
			.await?;
		Ok(())
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Timeline {
	pub feed: RwSignal<Vec<String>>,
	pub next: RwSignal<String>,
}

impl Timeline {
	pub fn new(url: String) -> Self {
		let feed = create_rw_signal(vec![]);
		let next = create_rw_signal(url);
		Timeline { feed, next }
	}

	pub fn reset(&self, url: String) {
		self.feed.set(vec![]);
		self.next.set(url);
	}

	pub async fn more(&self, auth: Signal<Option<String>>) -> reqwest::Result<()> {
		let feed_url = self.next.get();
		let collection : serde_json::Value = Http::fetch(&feed_url, auth).await?;
		let activities : Vec<serde_json::Value> = collection
			.ordered_items()
			.collect();
	
		let mut feed = self.feed.get();
		let mut older = process_activities(activities, auth).await;
		feed.append(&mut older);
		self.feed.set(feed);

		if let Some(next) = collection.next().id() {
			self.next.set(next);
		}

		Ok(())
	}
}

async fn process_activities(
	activities: Vec<serde_json::Value>,
	auth: Signal<Option<String>>,
) -> Vec<String> {
	let mut sub_tasks = Vec::new();
	let mut gonna_fetch = BTreeSet::new();
	let mut out = Vec::new();

	for activity in activities {
		// save embedded object if present
		if let Some(object) = activity.object().get() {
			if let Some(object_uri) = object.id() {
				CACHE.put(object_uri.to_string(), object.clone());
			}
		} else { // try fetching it
			if let Some(object_id) = activity.object().id() {
				if !gonna_fetch.contains(&object_id) {
					gonna_fetch.insert(object_id.clone());
					sub_tasks.push(fetch_and_update("objects", object_id, auth));
				}
			}
		}
	
		// save activity, removing embedded object
		let object_id = activity.object().id();
		if let Some(activity_id) = activity.id() {
			out.push(activity_id.to_string());
			CACHE.put(
				activity_id.to_string(),
				activity.clone().set_object(apb::Node::maybe_link(object_id))
			);
		}
	
		if let Some(uid) = activity.actor().id() {
			if CACHE.get(&uid).is_none() && !gonna_fetch.contains(&uid) {
				gonna_fetch.insert(uid.clone());
				sub_tasks.push(fetch_and_update("users", uid, auth));
			}
		}
	}

	futures::future::join_all(sub_tasks).await;

	out
}

async fn fetch_and_update(kind: &'static str, id: String, auth: Signal<Option<String>>) {
	match Http::fetch(&Uri::api(kind, &id), auth).await {
		Ok(data) => CACHE.put(id, data),
		Err(e) => console_warn(&format!("could not fetch '{id}': {e}")),
	}
}
