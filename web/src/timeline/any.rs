use apb::{Collection, CollectionPage};
use leptos::{either::Either, prelude::*};
use uriproxy::UriClass;

use crate::Cache;

#[component]
pub fn Loadable<El, V>(
	base: String,
	element: El,
	#[prop(optional)] convert: Option<UriClass>,
	#[prop(default = true)] prefetch: bool,
) -> impl IntoView
where
	El: Send + Sync + Fn(crate::Doc) -> V + 'static,
	V: IntoView + 'static
{

	let class = convert.unwrap_or(UriClass::Object);
	let auth = use_context::<crate::Auth>().expect("missing auth context");
	let fun = std::sync::Arc::new(element);

	let (older_next, older_items) = crate::cache::TIMELINES.get(&base)
		.unwrap_or((Some(base.clone()), vec![]));

	let (next, set_next) = signal(older_next);
	let (items, set_items) = signal(older_items);
	let (loading, set_loading) = signal(false);

	// TODO having the seen set just once would be a great optimization, but then it becomes FnMut...
	// let mut seen: std::collections::HashSet<String> = std::collections::HashSet::default();

	// TODO it's a bit wasteful to clone the key for every load() invocation, but capturing it in the
	//      closure opens an industrial pipeline of worms so this will do for now
	let load = move |key: String| leptos::task::spawn_local(async move {
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

		let new_next = object.next().id().ok();

		set_next.set(new_next.clone());

		let mut previous = items.get_untracked();
		let mut seen: std::collections::HashSet<String> = std::collections::HashSet::from_iter(previous.clone());
		let mut sub_tasks_a = Vec::new();
		let mut sub_tasks_b = Vec::new();

		for node in object.ordered_items().flat() {
			let Ok(id) = node.id() else {
				tracing::warn!("skipping item without id: {node:?}");
				continue
			};

			let _id = id.clone();
			match node.into_inner() {
				Ok(doc) => { // we got it embedded, store it right away!
					crate::cache::OBJECTS.include(std::sync::Arc::new(doc));
					if prefetch {
						sub_tasks_a.push(crate::cache::OBJECTS.prefetch(_id, class, auth));
					}
				},
				Err(_) => { // we just got the id, try dereferencing it right now
					let id = id.clone();
					if prefetch {
						sub_tasks_a.push(crate::cache::OBJECTS.prefetch(_id, class, auth));
					} else {
						sub_tasks_b.push(async move { crate::cache::OBJECTS.resolve(&_id, class, auth).await });
					};
				},
			};

			if !seen.contains(&id) {
				seen.insert(id.clone());
				previous.push(id);
			}

		}

		futures::future::join_all(sub_tasks_a).await;
		futures::future::join_all(sub_tasks_b).await;

		crate::cache::TIMELINES.store(&key, (new_next, previous.clone()));

		set_items.set(previous);
		set_loading.set(false);
	});

	let _base = base.clone();
	if let Some(auto_scroll) = use_context::<Signal<bool>>() {
		let _ = Effect::watch(
			move || auto_scroll.get(),
			move |at_end, _, _| if *at_end { load(_base.clone()) },
			false,
		);
	}

	load(base.clone());

	let _base = base.clone();
	view! {
		<div>
			<For
				each=move || items.get()
				key=|id: &String| id.clone()
				children=move |id: String| match crate::cache::OBJECTS.get(&id) {
					Some(obj) => Either::Left(fun(obj)),
					None => Either::Right(view!{ <p>{id}</p> }),
				}
			/>

			{move || if loading.get() {
				Some(Either::Left(view! {
					<div class="center mt-1 mb-1" >
						<button type="button" disabled>"loading "<span class="dots"></span></button>
					</div>
				}))
			} else if next.with(|x| x.is_some()) {
				let _base = base.clone();
				Some(Either::Right(view! {
					<div class="center mt-1 mb-1" >
						<button type="button" on:click=move |_| load(_base.clone()) >"load more"</button>
					</div>
				}))
			} else {
				None
			}}
		</div>
	}
}
