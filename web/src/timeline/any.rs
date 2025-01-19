use apb::{Collection, CollectionPage};
use leptos::{either::Either, prelude::*};
use uriproxy::UriClass;

use crate::Cache;

#[component]
pub fn Loadable<El, V>(
	base: String,
	element: El,
	#[prop(optional)] convert: Option<UriClass>,
	children: Children,
) -> impl IntoView
where
	El: Send + Sync + Fn(crate::Object) -> V + 'static,
	V: IntoView + 'static
{

	let class = convert.unwrap_or(UriClass::Object);
	let auth = use_context::<crate::Auth>().expect("missing auth context");
	let fun = std::sync::Arc::new(element);

	let (next, set_next) = signal(Some(base));
	let (items, set_items) = signal(vec![]);
	let (loading, set_loading) = signal(false);

	// TODO having it just once would be a great optimization, but then it becomes FnMut...
	// let mut seen: std::collections::HashSet<String> = std::collections::HashSet::default();

	let load = move || leptos::task::spawn_local(async move {
		// this concurrency is rather fearful honestly
		if loading.get_untracked() { return }
		set_loading.set(true);
		let Some(url) = next.get_untracked() else {
			set_loading.set(false);
			return
		};

		let object = match crate::Http::fetch::<serde_json::Value>(&url, auth).await {
			Ok(x) => x,
			Err(e) => {
				tracing::error!("could not fetch items ({url}): {e} -- {e:?}");
				set_next.set(None);
				set_loading.set(false);
				return;
			},
		};

		if let Ok(n) = object.next().id() {
			set_next.set(Some(n));
		} else {
			set_next.set(None);
		}

		let mut previous = items.get_untracked();
		let mut seen: std::collections::HashSet<String> = std::collections::HashSet::from_iter(previous.clone());
		let mut sub_tasks = Vec::new();

		for node in object.ordered_items().flat() {
			let Ok(id) = node.id() else {
				tracing::warn!("skipping item without id: {node:?}");
				continue
			};

			match node.into_inner() {
				Ok(doc) => { // we got it embedded, store it right away!
					crate::cache::OBJECTS.store(&id, std::sync::Arc::new(doc));
				},
				Err(_) => { // we just got the id, try dereferencing it right now
					let id = id.clone();
					sub_tasks.push(async move { crate::cache::OBJECTS.resolve(&id, class, auth).await });
				},
			};

			if !seen.contains(&id) {
				seen.insert(id.clone());
				previous.push(id);
			}
		}

		futures::future::join_all(sub_tasks).await;

		set_items.set(previous);
		set_loading.set(false);
	});

	if let Some(auto_scroll) = use_context::<Signal<bool>>() {
		let _ = Effect::watch(
			move || auto_scroll.get(),
			move |at_end, _, _| if *at_end { load() },
			false,
		);
	}

	load();

	view! {
		{children()}
		<For
			each=move || items.get()
			key=|id: &String| id.clone()
			children=move |id: String| crate::cache::OBJECTS.get(&id).map(|obj| fun(obj))
		/>

		{move || if loading.get() {
			Some(Either::Left(view! {
				<div class="center mt-1 mb-1" >
					<button type="button" disabled>"loading "<span class="dots"></span></button>
				</div>
			}))
		} else if next.get().is_some() {
			Some(Either::Right(view! {
				<div class="center mt-1 mb-1" >
					<button type="button" on:click=move |_| load() >"load more"</button>
				</div>
			}))
		} else {
			None
		}}
	}
}
