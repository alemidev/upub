pub mod feed;
pub mod thread;

use std::{collections::BTreeSet, pin::Pin, sync::Arc};

use apb::{field::OptionalString, Activity, ActivityMut, Actor, Base, Object};
use leptos::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Timeline {
	pub feed: RwSignal<Vec<String>>,
	pub next: RwSignal<String>,
	pub over: RwSignal<bool>,
	pub loading: RwSignal<bool>,
}

impl Timeline {
	pub fn new(url: String) -> Self {
		let feed = create_rw_signal(vec![]);
		let next = create_rw_signal(url);
		let over = create_rw_signal(false);
		let loading = create_rw_signal(false);
		Timeline { feed, next, over, loading }
	}

	pub fn len(&self) -> usize {
		self.feed.get().len()
	}

	pub fn is_empty(&self) -> bool {
		self.feed.get().is_empty()
	}

	pub fn reset(&self, url: Option<String>) {
		self.feed.set(vec![]);
		self.over.set(false);
		if let Some(url) = url {
			self.next.set(url);
		}
	}

	pub fn refresh(&self, auth: Auth, config: Signal<crate::Config>) {
		self.reset(
			self.next
				.get_untracked()
				.split('?')
				.next()
				.map(|x| x.to_string())
		);
		self.spawn_more(auth, config);
	}

	pub fn spawn_more(&self, auth: Auth, config: Signal<crate::Config>) {
		let _self = *self;
		spawn_local(async move {
			_self.more(auth, config).await
		});
	}

	pub fn loading(&self) -> bool {
		self.loading.get_untracked()
	}

	pub async fn more(&self, auth: Auth, config: Signal<crate::Config>) {
		if self.loading.get_untracked() { return }
		if self.over.get_untracked() { return }
		self.loading.set(true);
		let res = self.load_more(auth, config).await;
		self.loading.set(false);
		if let Err(e) = res {
			tracing::error!("failed loading posts for timeline: {e}");
		}
	}

	pub async fn load_more(&self, auth: Auth, config: Signal<crate::Config>) -> reqwest::Result<()> {
		use apb::{Collection, CollectionPage};

		let mut feed_url = self.next.get_untracked();
		if !config.get_untracked().filters.replies {
			feed_url = if feed_url.contains('?') {
				feed_url + "&replies=false"
			} else {
				feed_url + "?replies=false"
			};
		}
		let collection : serde_json::Value = Http::fetch(&feed_url, auth).await?;
		let activities : Vec<serde_json::Value> = collection
			.ordered_items()
			.collect();
	
		let mut feed = self.feed.get_untracked();
		let mut older = process_activities(activities, auth)
			.await
			.into_iter()
			.filter(|x| !feed.contains(x))
			.collect();
		feed.append(&mut older);
		self.feed.set(feed);

		if let Ok(next) = collection.next().id() {
			self.next.set(next.to_string());
		} else {
			self.over.set(true);
		}

		Ok(())
	}
}

// TODO ughhh this shouldn't be here if its pub!!!
pub async fn process_activities(activities: Vec<serde_json::Value>, auth: Auth) -> Vec<String> {
	let mut sub_tasks : Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = Vec::new();
	let mut gonna_fetch = BTreeSet::new();
	let mut actors_seen = BTreeSet::new();
	let mut out = Vec::new();

	for activity in activities {
		let activity_type = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
		// save embedded object if present
		if let Some(object) = activity.object().get() {
			// also fetch actor attributed to
			if let Some(attributed_to) = object.attributed_to().id().str() {
				actors_seen.insert(attributed_to);
			}
			if let Ok(object_uri) = object.id() {
				cache::OBJECTS.store(object_uri, Arc::new(object.clone()));
			} else {
				tracing::warn!("embedded object without id: {object:?}");
			}
		} else { // try fetching it
			if let Some(object_id) = activity.object().id().str() {
				if !gonna_fetch.contains(&object_id) {
					let fetch_kind = match activity_type {
						apb::ActivityType::Follow => U::Actor,
						_ => U::Object,
					};
					gonna_fetch.insert(object_id.clone());
					sub_tasks.push(Box::pin(fetch_and_update_with_user(fetch_kind, object_id, auth)));
				}
			}
		}
	
		// save activity, removing embedded object
		let object_id = activity.object().id().str();
		if let Some(activity_id) = activity.id().str() {
			out.push(activity_id.to_string());
			cache::OBJECTS.store(
				&activity_id,
				Arc::new(activity.clone().set_object(apb::Node::maybe_link(object_id)))
			);
		} else if let Some(object_id) = activity.object().id().str() {
			out.push(object_id);
		}

		if let Some(uid) = activity.attributed_to().id().str() {
			if cache::OBJECTS.get(&uid).is_none() && !gonna_fetch.contains(&uid) {
				gonna_fetch.insert(uid.clone());
				sub_tasks.push(Box::pin(fetch_and_update(U::Actor, uid, auth)));
			}
		}
	
		if let Some(uid) = activity.actor().id().str() {
			if cache::OBJECTS.get(&uid).is_none() && !gonna_fetch.contains(&uid) {
				gonna_fetch.insert(uid.clone());
				sub_tasks.push(Box::pin(fetch_and_update(U::Actor, uid, auth)));
			}
		}
	}

	for user in actors_seen {
		sub_tasks.push(Box::pin(fetch_and_update(U::Actor, user, auth)));
	}

	futures::future::join_all(sub_tasks).await;

	out
}

async fn fetch_and_update(kind: U, id: String, auth: Auth) {
	match Http::fetch::<serde_json::Value>(&Uri::api(kind, &id, false), auth).await {
		Err(e) => console_warn(&format!("could not fetch '{id}': {e}")),
		Ok(data) => {
			if data.actor_type().is_ok() {
				if let Some(url) = data.url().id().str() {
					cache::WEBFINGER.store(&id, url);
				}
			}
			cache::OBJECTS.store(&id, Arc::new(data));
		},
	}
}

async fn fetch_and_update_with_user(kind: U, id: String, auth: Auth) {
	fetch_and_update(kind, id.clone(), auth).await;
	if let Some(obj) = cache::OBJECTS.get(&id) {
		if let Some(actor_id) = match kind {
			U::Object => obj.attributed_to().id().str(),
			U::Activity => obj.actor().id().str(),
			U::Actor => None,
			U::Hashtag => None,
		} {
			fetch_and_update(U::Actor, actor_id, auth).await;
		}
	}
}
