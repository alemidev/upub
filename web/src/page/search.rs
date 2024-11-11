use std::sync::Arc;

use apb::Collection;
use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn SearchPage() -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");

	let query = Signal::derive(||
		use_query_map().with(|x| x.get("q").cloned().unwrap_or_default())
	);

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
			async move {
				let items = Http::fetch::<serde_json::Value>(&search, auth).await.ok()?;
				Some(
					crate::timeline::process_activities(
						items
							.ordered_items()
							.flat()
							.into_iter()
							.filter_map(|x| x.extract())
							.collect(),
						auth
					).await
				)
			}
		}
	);


	view! {

		<blockquote class="mt-3 mb-3">
			<details open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">actor</code>
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
					<code class="cw center color ml-s w-100">object</code>
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

		<blockquote>
			<details open>
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
			<details open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">full text</code>
				</summary>
				<div class="pb-1">
					{move || match text_search.get() {
						None => Some(view! { <p class="center"><small>searching...</small></p> }.into_view()),
						Some(None) => None,
						Some(Some(items)) => Some(view! {
							// TODO ughhh too many clones
							<For
								each=move || items.clone()
								key=|id| id.clone()
								children=move |item| {
									cache::OBJECTS.get(&item)
										.map(|x| view! { <Item item=x always=true /> }.into_view())
								}
							/ >
						}.into_view())
					}}
				</div>
			</details>
		</blockquote>
	}
}
