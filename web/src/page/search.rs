use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use crate::prelude::*;

#[component]
pub fn SearchPage() -> impl IntoView {
	let (query, set_query) = signal("".to_string());

	use_query_map().with(|q| {
		if let Some(q) = q.get("q") {
			set_query.set(q);
		}
	});

	view! {

		<a href={format!("/web/tags/{}", query.get())}>#{query.get()}</a>

		<blockquote class="mt-3 mb-3">
			<details class="cw" open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">actors</code>
				</summary>
				<div class="pb-1">
					<Loadable
						base=format!("{URL_BASE}/search/actors?q={}", query.get())
						convert=U::Actor
						element=|obj| view! { <ActorBanner object=obj /> }
					/>
				</div>
			</details>
		</blockquote>

		<blockquote class="mt-3 mb-3">
			<details class="cw" open>
				<summary class="mb-2">
					<code class="cw center color ml-s w-100">objects</code>
				</summary>
				<div class="pb-1">
					<Loadable
						base=format!("{URL_BASE}/search/objects?q={}", query.get())
						element=|obj| view! { <Item item=obj sep=true /> }
					/>
				</div>
			</details>
		</blockquote>

	}
}
