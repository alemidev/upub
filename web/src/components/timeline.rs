use std::{collections::BTreeSet, pin::Pin};

use apb::{Activity, Base, Object};
use leptos::*;
use crate::prelude::*;

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
		use apb::{Collection, CollectionPage};

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

#[component]
pub fn TimelineFeed(tl: Timeline) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<For
			each=move || tl.feed.get()
			key=|k| k.to_string()
			children=move |id: String| {
				match CACHE.get(&id) {
					Some(item) => match item.base_type() {
						Some(apb::BaseType::Object(apb::ObjectType::Activity(_))) => {
							let object_id = item.object().id().unwrap_or_default();
							let object = CACHE.get(&object_id).map(|obj| {
								view! { <ObjectInline object=obj /> }
							});
							view! {
								<ActivityLine activity=item />
								{object}
								<hr/ >
							}.into_view()
						},
						Some(apb::BaseType::Object(apb::ObjectType::Note)) => view! {
								<Object object=item.clone() />
								<hr/ >
							}.into_view(),
						_ => view! { <p><code>type not implemented</code></p><hr /> }.into_view(),
					},
					None => view! {
						<p><code>{id}</code>" "[<a href={uri}>go</a>]</p>
					}.into_view(),
				}
			}
		/ >
		<div class="center mt-1 mb-1" >
			<button type="button"
				on:click=move |_| {
					spawn_local(async move {
						if let Err(e) = tl.more(auth).await {
							tracing::error!("error fetching more items for timeline: {e}");
						}
					})
				}
			>more</button>
		</div>
	}
}

async fn process_activities(
	activities: Vec<serde_json::Value>,
	auth: Signal<Option<String>>,
) -> Vec<String> {
	use apb::ActivityMut;
	let mut sub_tasks : Vec<Pin<Box<dyn futures::Future<Output = ()>>>> = Vec::new();
	let mut gonna_fetch = BTreeSet::new();
	let mut actors_seen = BTreeSet::new();
	let mut out = Vec::new();

	for activity in activities {
		// save embedded object if present
		if let Some(object) = activity.object().get() {
			// also fetch actor attributed to
			if let Some(attributed_to) = object.attributed_to().id() {
				actors_seen.insert(attributed_to);
			}
			if let Some(object_uri) = object.id() {
				CACHE.put(object_uri.to_string(), object.clone());
			} else {
				tracing::warn!("embedded object without id: {object:?}");
			}
		} else { // try fetching it
			if let Some(object_id) = activity.object().id() {
				if !gonna_fetch.contains(&object_id) {
					gonna_fetch.insert(object_id.clone());
					sub_tasks.push(Box::pin(fetch_and_update_with_user(FetchKind::Object, object_id, auth)));
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
				sub_tasks.push(Box::pin(fetch_and_update(FetchKind::User, uid, auth)));
			}
		}
	}

	for user in actors_seen {
		sub_tasks.push(Box::pin(fetch_and_update(FetchKind::User, user, auth)));
	}

	futures::future::join_all(sub_tasks).await;

	out
}

async fn fetch_and_update(kind: FetchKind, id: String, auth: Signal<Option<String>>) {
	match Http::fetch(&Uri::api(kind, &id, false), auth).await {
		Ok(data) => CACHE.put(id, data),
		Err(e) => console_warn(&format!("could not fetch '{id}': {e}")),
	}
}

async fn fetch_and_update_with_user(kind: FetchKind, id: String, auth: Signal<Option<String>>) {
	fetch_and_update(kind.clone(), id.clone(), auth).await;
	if let Some(obj) = CACHE.get(&id) {
		if let Some(actor_id) = match kind {
			FetchKind::Object => obj.attributed_to().id(),
			FetchKind::Activity => obj.actor().id(),
			FetchKind::User | FetchKind::Context => None,
		} {
			fetch_and_update(FetchKind::User, actor_id, auth).await;
		}
	}
}
