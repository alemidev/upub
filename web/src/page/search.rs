use std::sync::Arc;

use apb::{Base, Collection};
use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn SearchPage() -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");

	let user = create_local_resource(
		move || use_query_map().get().get("q").cloned().unwrap_or_default(),
		move |q| {
			let user_fetch = Uri::api(U::Actor, &q, true);
			async move { Some(Arc::new(Http::fetch::<serde_json::Value>(&user_fetch, auth).await.ok()?)) }
		}
	);
	
	let object = create_local_resource(
		move || use_query_map().get().get("q").cloned().unwrap_or_default(),
		move |q| {
			let object_fetch = Uri::api(U::Object, &q, true);
			async move { Some(Arc::new(Http::fetch::<serde_json::Value>(&object_fetch, auth).await.ok()?)) }
		}
	);

	let text_search = create_local_resource(
		move || use_query_map().get().get("q").cloned().unwrap_or_default(),
		move |q| {
			let search = format!("{URL_BASE}/search?q={q}");
			async move { Http::fetch::<serde_json::Value>(&search, auth).await.ok() }
		}
	);

	view! {
		<blockquote class="mt-3 mb-3">
			<details open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">users</code>
				</summary>
				<div class="pb-1">
				{move || match user.get() {
					None => view! { <p class="center"><small>searching...</small></p> },
					Some(None) => view! { <p class="center"><code>N/A</code></p> },
					Some(Some(u)) => view! { <p><ActorBanner object=u /></p> },
				}}
				</div>
			</details>
		</blockquote>

		<blockquote class="mt-3 mb-3">
			<details open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">objects</code>
				</summary>
				<div class="pb-1">
				{move || match object.get() {
					None => view! { <p class="center"><small>searching...</small></p> },
					Some(None) => view!{ <p class="center"><code>N/A</code></p> },
					Some(Some(o)) => view! { <p><Object object=o /></p> },
				}}
				</div>
			</details>
		</blockquote>

		{move || match text_search.get() {
			None => Some(view! { <p class="center"><small>searching...</small></p> }.into_view()),
			Some(None) => None,
			Some(Some(items)) => Some(view! {
				// TODO this is jank af! i should do the same thing i do for timelines, aka first process
				//      all items and store in cache and then pass a vec of strings here!!!
				<For
					each=move || items.ordered_items()
					key=|item| item.id().unwrap_or_default().to_string()
					children=move |item| {
						view! {
							<Item item=item.into() />
							<hr />
						}.into_view()
					}
				/ >
			}.into_view())
		}}
	}
}
