use leptos::{either::Either, prelude::*};
use leptos_router::hooks::query_signal;
use crate::prelude::*;

#[component]
pub fn SearchPage() -> impl IntoView {
	let (query, _set_query) = query_signal::<String>("q");

	move || match query.get() {
		Some(q) => Either::Left(view! {
			<p class="ml-3">
				<a href={format!("/web/tags/{q}")}>#{q.clone()}</a>
			</p>

			<blockquote class="mt-3 mb-3">
				<details class="cw" open>
					<summary>
						<code class="cw center color ml-s w-100">actors</code>
					</summary>
					<div class="pb-1 pt-1 pl-2">
						<Loadable
							base=format!("{URL_BASE}/search/actors?q={q}")
							convert=U::Actor
							element=|obj| view! { <ActorBanner object=obj /> }
						/>
					</div>
				</details>
			</blockquote>

			<blockquote class="mt-3 mb-3">
				<details class="cw" open>
					<summary>
						<code class="cw center color ml-s w-100">objects</code>
					</summary>
					<div class="pb-1 pt-1">
						<Loadable
							base=format!("{URL_BASE}/search/objects?q={q}")
							element=|obj| view! { <Item item=obj sep=true always=true /> }
							replies=true
						/>
					</div>
				</details>
			</blockquote>
		}),
		None => Either::Right(view! {
			<code class="center color cw ma-3">no search query given</code>
		}),
	}
}
