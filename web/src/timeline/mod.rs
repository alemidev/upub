use std::sync::Arc;

use apb::{Activity, Base, Collection, CollectionPage, Object};
use leptos::{either::Either, prelude::*};
use uriproxy::UriClass;

use crate::{Auth, Cache};


// TODO would be cool if "element" was passed as children() somehow
// TODO "thread" is a bit weird, maybe better to make two distinct components?
#[component]
pub fn Loadable<El, V>(
	base: String,
	element: El,
	#[prop(default = UriClass::Activity)] convert: UriClass,
	#[prop(default = true)] preload: bool,
	#[prop(default = false)] replies: bool,
	#[prop(optional)] thread: Option<String>,
) -> impl IntoView
where
	El: Send + Sync + Fn(crate::Doc) -> V + 'static,
	V: IntoView + 'static
{

	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	let auth = use_context::<crate::Auth>().expect("missing auth context");
	let fun = Arc::new(element);

	let (older_next, older_items) = crate::cache::TIMELINES.get(&base)
		.unwrap_or((Some(base.clone()), vec![]));

	let (next, set_next) = signal(older_next);
	let (items, set_items) = signal(older_items);
	let (loading, set_loading) = signal(false);

	// TODO having the seen set just once would be a great optimization, but then it becomes FnMut...
	// let mut seen: std::collections::HashSet<String> = std::collections::HashSet::default();

	// TODO it's a bit wasteful to clone the key for every load() invocation, but capturing it in the
	//      closure opens an industrial pipeline of worms so this will do for now
	let load = move |key: String| {
		if loading.get_untracked() { return }
		set_loading.set(true);
		leptos::task::spawn_local(async move {
			// this concurrency is rather fearful honestly
			let Some(mut url) = next.get_untracked() else {
				set_loading.set(false);
				return
			};

			// TODO a more elegant way to do this!!
			if !replies && !config.get_untracked().filters.replies {
				if url.contains('?') {
					url += "&replies=false";
				} else {
					url += "?replies=false";
				}
			}

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

			let store = process_activities(
				object,
				items.get_untracked(),
				preload,
				convert,
				auth
			).await;

			crate::cache::TIMELINES.store(&key, (new_next, store.clone()));

			set_items.set(store);
			set_loading.set(false);
		})
	};

	let auto_scroll = use_context::<Signal<bool>>().expect("missing auto-scroll signal");
	let _base = base.clone();
	let _ = Effect::watch(
		move || auto_scroll.get(),
		move |at_end, _, _| if *at_end && config.get_untracked().infinite_scroll {
			load(_base.clone())
		},
		false,
	);

	let reload = use_context::<ReadSignal<()>>().expect("missing reload signal");
	let _base = base.clone();
	let _ = Effect::watch(
		move || reload.get(),
		move |_, _, _| {
			set_items.set(vec![]);
			set_next.set(Some(_base.clone()));
			crate::cache::TIMELINES.invalidate(&_base);
			load(_base.clone());
		},
		false
	);

	if items.get_untracked().is_empty() {
		load(base.clone());
	}

	let _base = base.clone();
	view! {
		<div>
			{if let Some(root) = thread {
				Either::Left(view! { <FeedRecursive items=items root=root element=fun /> })
			} else {
				Either::Right(view! { <FeedLinear items=items element=fun /> })
			}}

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

#[component]
fn FeedLinear<El, V>(items: ReadSignal<Vec<String>>, element: Arc<El>) -> impl IntoView
where
	El: Send + Sync + Fn(crate::Doc) -> V + 'static,
	V: IntoView + 'static
{
	view! {
		<For
			each=move || items.get()
			key=|id: &String| id.clone()
			children=move |id: String| match crate::cache::OBJECTS.get(&id) {
				Some(obj) => Either::Left(element(obj)),
				None => Either::Right(view!{ <p><code class="center color cw">{id}</code></p> }),
			}
		/>
	}
}

#[component]
fn FeedRecursive<El, V>(items: ReadSignal<Vec<String>>, root: String, element: Arc<El>) -> impl IntoView
where
	El: Send + Sync + Fn(crate::Doc) -> V + 'static,
	V: IntoView + 'static
{
	let root_values = move || items.get()
		.into_iter()
		.filter_map(|x| {
			let document = crate::cache::OBJECTS.get(&x)?;
			let (oid, reply) = match document.object_type().ok()? {
				// if it's a create, get and check created object: does it reply to root?
				apb::ObjectType::Activity(apb::ActivityType::Create) => {
					let object = crate::cache::OBJECTS.get(&document.object().id().ok()?)?;
					(object.id().ok()?, object.in_reply_to().id().ok()?)
				},

				// if it's a raw note, directly check if it replies to root
				apb::ObjectType::Note => (document.id().ok()?, document.in_reply_to().id().ok()?),

				// if it's anything else, check if it relates to root, maybe like or announce?
				_ => (document.id().ok()?, document.object().id().ok()?),
			};
			if reply == root {
				Some((oid, document))
			} else {
				None
			}
		})
		.collect::<Vec<(String, crate::Doc)>>();

	view! {
		<For
			each=root_values
			key=|(id, _obj)| id.clone()
			children=move |(id, obj)|
				view! {
					<details class="thread context depth-r" open>
						<summary>
							{element(obj)}
						</summary>
						<div class="depth-r">
							<FeedRecursive items=items root=id element=element.clone() />
						</div>
					</details>
				}
		/ >
	}.into_any()
}

pub async fn process_activities(
	object: serde_json::Value,
	mut store: Vec<String>,
	preload: bool,
	convert: UriClass,
	auth: Auth,
) -> Vec<String> {
	let mut seen: std::collections::HashSet<String> = std::collections::HashSet::from_iter(store.clone());
	let mut sub_tasks = Vec::new();

	for node in object.ordered_items().flat() {
		let mut added_something = false;
		if let Ok(id) = node.id() {
			added_something = true;
			if !seen.contains(&id) {
				seen.insert(id.clone());
				store.push(id.clone());
			}

			if preload {
				sub_tasks.push(crate::cache::OBJECTS.preload(id, convert, auth));
			}
		}

		if let Ok(doc) = node.into_inner() {
			// TODO this is weird because we manually go picking up the inner object
			//      worse: objects coming from fetches get stitched in timelines with empty shell
			//      "View" activities which don't have an id. in such cases we want the inner object to
			//      appear on our timelines, so we must do what we would do for the activity (but we
			//      didn't do) for our inner object, and also we can be pretty sure it's an object so
			//      override class
			if let Ok(sub_doc) = doc.object().into_inner() {
				if let Ok(sub_id) = sub_doc.id() {
					if !added_something && !seen.contains(&sub_id) {
						seen.insert(sub_id.clone());
						store.push(sub_id.clone());
					}
					if preload {
						sub_tasks.push(crate::cache::OBJECTS.preload(sub_id, UriClass::Object, auth));
					}
				}
			}
			crate::cache::OBJECTS.include(Arc::new(doc));
		};
	}

	futures::future::join_all(sub_tasks).await;

	store
}
