use std::sync::Arc;

use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn SearchPage() -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");

	let user = create_local_resource(
		move || use_query_map().get().get("q").cloned().unwrap_or_default(),
		move |q| {
			let user_fetch = Uri::api(U::User, &q, true);
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

	view! {
		<Breadcrumb>search</Breadcrumb>
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
	}
}
