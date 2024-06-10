use std::sync::Arc;

use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use apb::Object;

#[component]
pub fn UserPage() -> impl IntoView {
	let params = use_params_map();
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let auth = use_context::<Auth>().expect("missing auth context");
	let id = params.get()
		.get("id")
		.cloned()
		.unwrap_or_default();
	let uid = uriproxy::uri(URL_BASE, uriproxy::UriClass::Actor, &id);
	let actor = create_local_resource(
		move || params.get().get("id").cloned().unwrap_or_default(),
		move |id| {
			async move {
				let tl_url = format!("{}/outbox/page", Uri::api(U::Actor, &id, false));
				if !feeds.user.next.get_untracked().starts_with(&tl_url) {
					feeds.user.reset(Some(tl_url));
				}
				match CACHE.get(&Uri::full(U::Actor, &id)) {
					Some(x) => Some(x.clone()),
					None => {
						let user : serde_json::Value = Http::fetch(&Uri::api(U::Actor, &id, true), auth).await.ok()?;
						let user = Arc::new(user);
						CACHE.put(Uri::full(U::Actor, &id), user.clone());
						Some(user)
					},
				}
			}
		}
	);
	view! {
		<div>
			<Breadcrumb back=true >
				actors::view
				<a
					class="clean ml-1" href="#"
					class:hidden=move || feeds.user.is_empty()
					on:click=move |_| {
						feeds.user.reset(Some(feeds.user.next.get().split('?').next().unwrap_or_default().to_string()));
						feeds.user.more(auth);
				}><span class="emoji">
					"\u{1f5d8}"
				</span></a>
			</Breadcrumb>
			<div>
				{move || {
					let uid = uid.clone();
					match actor.get() {
						None => view! { <p class="center">loading...</p> }.into_view(),
						Some(None) => {
							view! { <p class="center"><code>loading failed</code><sup><small><a class="clean" href={uid} target="_blank">"↗"</a></small></sup></p> }.into_view()
						},
						Some(Some(object)) => {
							view! {
								<div class="ml-3 mr-3">
									<ActorHeader object=object.clone() />
									<p class="ml-2 mt-1 center" inner_html={mdhtml::safe_html(object.summary().unwrap_or_default())}></p>
								</div>
								<TimelineFeed tl=feeds.user />
							}.into_view()
						},
					}
				}}
			</div>
		</div>
	}
}
