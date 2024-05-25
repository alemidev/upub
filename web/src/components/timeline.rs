use std::{collections::BTreeSet, pin::Pin, sync::Arc};

use apb::{Activity, ActivityMut, Base, Object};
use leptos::*;
use leptos_use::{signal_debounced, signal_throttled, use_display_media, use_document_visibility, use_element_size, use_infinite_scroll_with_options, use_scroll, use_scroll_with_options, use_window, use_window_scroll, UseDisplayMediaReturn, UseElementSizeReturn, UseInfiniteScrollOptions, UseScrollOptions, UseScrollReturn};
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

	pub fn reset(&self, url: String) {
		self.feed.set(vec![]);
		self.next.set(url);
		self.over.set(false);
	}

	pub async fn more(&self, auth: Auth) -> reqwest::Result<()> {
		self.loading.set(true);
		let res = self.more_inner(auth).await;
		self.loading.set(false);
		res
	}

	async fn more_inner(&self, auth: Auth) -> reqwest::Result<()> {
		use apb::{Collection, CollectionPage};

		let feed_url = self.next.get_untracked();
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

		if let Some(next) = collection.next().id() {
			self.next.set(next);
		} else {
			self.over.set(true);
		}

		Ok(())
	}
}

#[component]
pub fn TimelineRepliesRecursive(tl: Timeline, root: String) -> impl IntoView {
	let root_values = move || tl.feed
		.get()
		.into_iter()
		.filter_map(|x| CACHE.get(&x))
		.filter(|x| match x.object_type() {
			Some(apb::ObjectType::Activity(apb::ActivityType::Create)) => {
				let Some(oid) = x.object().id() else { return false; };
				let Some(object) = CACHE.get(&oid) else { return false; };
				let Some(reply) = object.in_reply_to().id() else { return false; };
				reply == root
			},
			Some(apb::ObjectType::Activity(_)) => x.object().id().map(|o| o == root).unwrap_or(false),
			_ => x.in_reply_to().id().map(|r| r == root).unwrap_or(false),
		})
		.collect::<Vec<crate::Object>>();

	view! {
		<For
			each=root_values
			key=|k| k.id().unwrap_or_default().to_string()
			children=move |obj: crate::Object| {
				let oid = obj.id().unwrap_or_default().to_string();
				view! {
					<div class="context depth-r">
						<Item item=obj />
						<div class="depth-r">
							<TimelineRepliesRecursive tl=tl root=oid />
						</div>
					</div>
				}
			}
		/ >
	}
}

#[component]
pub fn TimelineReplies(tl: Timeline, root: String) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");

	view! {
		<div>
			<TimelineRepliesRecursive tl=tl root=root />
		</div>
		<div class="center mt-1 mb-1" class:hidden=tl.over >
			<button type="button"
				prop:disabled=tl.loading 
				on:click=move |_| {
					spawn_local(async move {
						if let Err(e) = tl.more(auth).await {
							tracing::error!("error fetching more items for timeline: {e}");
						}
					})
				}
			>
				{move || if tl.loading.get() {
					view! { "loading"<span class="dots"></span> }.into_view()
				} else { "more".into_view() }}
			</button>
		</div>
	}
}

#[component]
pub fn TimelineFeed(tl: Timeline) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	// double view height: preload when 1 screen away
	let view_height = 2.0 * window()
		.inner_height()
		.map_or(500.0, |v| v.as_f64().unwrap_or_default());
	let scroll_ref = create_node_ref();
	let UseElementSizeReturn { width: _w, height } = use_element_size(scroll_ref);
	let (_x, scroll) = use_window_scroll();
	let scroll_debounced = signal_throttled(scroll, 500.0);
	let _auto_loader = create_local_resource(
		move || (scroll_debounced.get(), height.get()),
		move |(s, h)| async move {
			if !config.get().infinite_scroll { return }
			if !tl.loading.get() && h - s < view_height {
				if let Err(e) = tl.more(auth).await {
					tracing::error!("auto load failed: {e}");
				}
			}
		},
	);
	view! {
		<div ref=scroll_ref>
			<For
				each=move || tl.feed.get()
				key=|k| k.to_string()
				children=move |id: String| {
					match CACHE.get(&id) {
						Some(i) => view! {
							<Item item=i sep=true />
						}.into_view(),
						None => view! {
							<p><code>{id}</code>" "[<a href={uri}>go</a>]</p>
							<hr />
						}.into_view(),
					}
				}
			/ >
		</div>
		<div class="center mt-1 mb-1" class:hidden=tl.over >
			<button type="button"
				prop:disabled=tl.loading 
				on:click=move |_| load_more(tl, auth)
			>
				{move || if tl.loading.get() {
					view! { "loading"<span class="dots"></span> }.into_view()
				} else { "more".into_view() }}
			</button>
		</div>
	}
}

fn load_more(tl: Timeline, auth: Auth) {
	if !tl.loading.get() {
		spawn_local(async move {
			if let Err(e) = tl.more(auth).await {
				tracing::error!("error fetching more items for timeline: {e}");
			}
		})
	}
}

async fn process_activities(activities: Vec<serde_json::Value>, auth: Auth) -> Vec<String> {
	let mut sub_tasks : Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = Vec::new();
	let mut gonna_fetch = BTreeSet::new();
	let mut actors_seen = BTreeSet::new();
	let mut out = Vec::new();

	for activity in activities {
		let activity_type = activity.activity_type().unwrap_or(apb::ActivityType::Activity);
		// save embedded object if present
		if let Some(object) = activity.object().get() {
			// also fetch actor attributed to
			if let Some(attributed_to) = object.attributed_to().id() {
				actors_seen.insert(attributed_to);
			}
			if let Some(object_uri) = object.id() {
				CACHE.put(object_uri.to_string(), Arc::new(object.clone()));
			} else {
				tracing::warn!("embedded object without id: {object:?}");
			}
		} else { // try fetching it
			if let Some(object_id) = activity.object().id() {
				if !gonna_fetch.contains(&object_id) {
					let fetch_kind = match activity_type {
						apb::ActivityType::Follow => U::User,
						_ => U::Object,
					};
					gonna_fetch.insert(object_id.clone());
					sub_tasks.push(Box::pin(fetch_and_update_with_user(fetch_kind, object_id, auth)));
				}
			}
		}
	
		// save activity, removing embedded object
		let object_id = activity.object().id();
		if let Some(activity_id) = activity.id() {
			out.push(activity_id.to_string());
			CACHE.put(
				activity_id.to_string(),
				Arc::new(activity.clone().set_object(apb::Node::maybe_link(object_id)))
			);
		} else if let Some(object_id) = activity.object().id() {
			out.push(object_id);
		}

		if let Some(uid) = activity.attributed_to().id() {
			if CACHE.get(&uid).is_none() && !gonna_fetch.contains(&uid) {
				gonna_fetch.insert(uid.clone());
				sub_tasks.push(Box::pin(fetch_and_update(U::User, uid, auth)));
			}
		}
	
		if let Some(uid) = activity.actor().id() {
			if CACHE.get(&uid).is_none() && !gonna_fetch.contains(&uid) {
				gonna_fetch.insert(uid.clone());
				sub_tasks.push(Box::pin(fetch_and_update(U::User, uid, auth)));
			}
		}
	}

	for user in actors_seen {
		sub_tasks.push(Box::pin(fetch_and_update(U::User, user, auth)));
	}

	futures::future::join_all(sub_tasks).await;

	out
}

async fn fetch_and_update(kind: U, id: String, auth: Auth) {
	match Http::fetch(&Uri::api(kind, &id, false), auth).await {
		Ok(data) => CACHE.put(id, Arc::new(data)),
		Err(e) => console_warn(&format!("could not fetch '{id}': {e}")),
	}
}

async fn fetch_and_update_with_user(kind: U, id: String, auth: Auth) {
	fetch_and_update(kind, id.clone(), auth).await;
	if let Some(obj) = CACHE.get(&id) {
		if let Some(actor_id) = match kind {
			U::Object => obj.attributed_to().id(),
			U::Activity => obj.actor().id(),
			U::User | U::Context => None,
		} {
			fetch_and_update(U::User, actor_id, auth).await;
		}
	}
}
