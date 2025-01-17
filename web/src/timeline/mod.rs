pub mod feed;
pub mod thread;

use std::{collections::BTreeSet, pin::Pin, sync::Arc};

use apb::{Activity, ActivityMut, Base, Object};
use leptos::prelude::*;
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
		let feed = RwSignal::new(vec![]);
		let next = RwSignal::new(url);
		let over = RwSignal::new(false);
		let loading = RwSignal::new(false);
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
		let url = url.unwrap_or_else(||
			self.next
				.get_untracked()
				.split('?')
				.next()
				.map(|x| x.to_string())
				.unwrap_or("".to_string())
		);
		self.next.set(url);
	}

	pub fn refresh(&self, auth: Auth, config: Signal<crate::Config>) {
		self.reset(None);
		self.spawn_more(auth, config);
	}

	pub fn spawn_more(&self, auth: Auth, config: Signal<crate::Config>) {
		let _self = *self;
		leptos::task::spawn_local(async move {
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
			.flat()
			.into_iter()
			.filter_map(|x| x.into_inner().ok())
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

// TODO fetching stuff is quite centralized in upub BE but FE has this mess of three functions
//      which interlock and are supposed to prime the global cache with everything coming from a
//      tl. can we streamline it a bit like in our BE? maybe some traits?? maybe reuse stuff???

// TODO ughhh this shouldn't be here if its pub!!!
pub async fn process_activities(activities: Vec<serde_json::Value>, auth: Auth) -> Vec<String> {
	let mut sub_tasks : Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = Vec::new();
	let mut gonna_fetch = BTreeSet::new();
	let mut actors_seen = BTreeSet::new();
	let mut out = Vec::new();

	for activity in activities {
		let activity_type = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
		// save embedded object if present
		if let Ok(object) = activity.object().inner() {
			// also fetch actor attributed to
			if let Ok(attributed_to) = object.attributed_to().id() {
				actors_seen.insert(attributed_to);
			}
			if let Ok(quote_id) = object.quote_url().id() {
				if !gonna_fetch.contains(&quote_id) {
					gonna_fetch.insert(quote_id.clone());
					sub_tasks.push(Box::pin(deep_fetch_and_update(U::Object, quote_id, auth)));
				}
			}
			if let Ok(object_uri) = object.id() {
				cache::OBJECTS.store(&object_uri, Arc::new(object.clone()));
			} else {
				tracing::warn!("embedded object without id: {object:?}");
			}
		} else { // try fetching it
			if let Ok(object_id) = activity.object().id() {
				if !gonna_fetch.contains(&object_id) {
					let fetch_kind = match activity_type {
						apb::ActivityType::Follow => U::Actor,
						_ => U::Object,
					};
					gonna_fetch.insert(object_id.clone());
					sub_tasks.push(Box::pin(deep_fetch_and_update(fetch_kind, object_id, auth)));
				}
			}
		}
	
		// save activity, removing embedded object
		let object_id = activity.object().id().ok();
		if let Ok(activity_id) = activity.id() {
			out.push(activity_id.to_string());
			cache::OBJECTS.store(
				&activity_id,
				Arc::new(activity.clone().set_object(apb::Node::maybe_link(object_id)))
			);
		} else if let Ok(object_id) = activity.object().id() {
			out.push(object_id);
		}

		if let Ok(uid) = activity.attributed_to().id() {
			if cache::OBJECTS.get(&uid).is_none() && !gonna_fetch.contains(&uid) {
				gonna_fetch.insert(uid.clone());
				sub_tasks.push(Box::pin(deep_fetch_and_update(U::Actor, uid, auth)));
			}
		}
	
		if let Ok(uid) = activity.actor().id() {
			if cache::OBJECTS.get(&uid).is_none() && !gonna_fetch.contains(&uid) {
				gonna_fetch.insert(uid.clone());
				sub_tasks.push(Box::pin(deep_fetch_and_update(U::Actor, uid, auth)));
			}
		}
	}

	for user in actors_seen {
		sub_tasks.push(Box::pin(deep_fetch_and_update(U::Actor, user, auth)));
	}

	futures::future::join_all(sub_tasks).await;

	out
}

async fn deep_fetch_and_update(kind: U, id: String, auth: Auth) {
	if let Some(obj) = cache::OBJECTS.resolve(&id, kind, auth).await {
		if let Ok(quote) = obj.quote_url().id() {
			cache::OBJECTS.resolve(&quote, U::Object, auth).await;
		}
		if let Ok(actor) = obj.actor().id() {
			cache::OBJECTS.resolve(&actor, U::Actor, auth).await;
		}
		if let Ok(attributed_to) = obj.attributed_to().id() {
			cache::OBJECTS.resolve(&attributed_to, U::Actor, auth).await;
		}
	}
}
