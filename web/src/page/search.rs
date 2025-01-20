use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use crate::prelude::*;

#[component]
pub fn SearchPage() -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");

	let query = Signal::derive(||
		use_query_map().with(|x| x.get("q").unwrap_or_default())
	);

	let user = LocalResource::new(
		move || {
			let q = use_query_map().get().get("q").unwrap_or_default();
			let user_fetch = Uri::api(U::Actor, &q, true);
			async move { Some(Arc::new(Http::fetch::<serde_json::Value>(&user_fetch, auth).await.ok()?)) }
		}
	);
	
	let object = LocalResource::new(
		move || {
			let q = use_query_map().get().get("q").unwrap_or_default();
			let object_fetch = Uri::api(U::Object, &q, true);
			async move { Some(Arc::new(Http::fetch::<serde_json::Value>(&object_fetch, auth).await.ok()?)) }
		}
	);

	let text_search = LocalResource::new(
		move || {
			let q = use_query_map().get().get("q").unwrap_or_default();
			let search = format!("{URL_BASE}/search?q={q}");
			async move {
				let document = Http::fetch::<serde_json::Value>(&search, auth).await.ok()?;
				Some(
					crate::timeline::process_activities(
						document,
						Vec::new(),
						true,
						uriproxy::UriClass::Object,
						auth,
					).await
				)
			}
		}
	);


	view! {

		<blockquote class="mt-3 mb-3">
			<details class="cw" open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">actor</code>
				</summary>
				<div class="pb-1">
				{move || match user.get().map(|x| x.take()) {
					None => view! { <p class="center"><small>searching...</small></p> }.into_any(),
					Some(None) => view! { <p class="center"><code>N/A</code></p> }.into_any(),
					Some(Some(u)) => view! { <p><ActorBanner object=u /></p> }.into_any(),
				}}
				</div>
			</details>
		</blockquote>

		<blockquote class="mt-3 mb-3">
			<details class="cw" open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">object</code>
				</summary>
				<div class="pb-1">
				{move || match object.get().map(|x| x.take()) {
					None => view! { <p class="center"><small>searching...</small></p> }.into_any(),
					Some(None) => view!{ <p class="center"><code>N/A</code></p> }.into_any(),
					Some(Some(o)) => view! { <p><Object object=o /></p> }.into_any(),
				}}
				</div>
			</details>
		</blockquote>

		<blockquote>
			<details class="cw" open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">hashtags</code>
				</summary>
				<div class="pb-1">
					<ul>
						<li><a href={format!("/web/tags/{}", query.get())}>#{query}</a></li>
					</ul>
				</div>
			</details>
		</blockquote>


		<blockquote class="mt-3 mb-3">
			<details class="cw" open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">full text</code>
				</summary>
				<div class="pb-1">
					{move || match text_search.get().map(|x| x.take()) {
						None => Some(view! { <p class="center"><small>searching...</small></p> }.into_any()),
						Some(None) => None,
						Some(Some(items)) => Some(view! {
							// TODO ughhh too many clones
							<For
								each=move || items.clone()
								key=|id| id.clone()
								children=move |item| {
									cache::OBJECTS.get(&item)
										.map(|x| view! { <Item item=x always=true /> }.into_any())
								}
							/ >
						}.into_any())
					}}
				</div>
			</details>
		</blockquote>
	}
}
