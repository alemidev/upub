use std::sync::Arc;

use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use apb::Collection;

#[component]
pub fn FollowPage(outgoing: bool) -> impl IntoView {
	let follow___ = if outgoing { "following" } else { "followers" };
	let params = use_params_map();
	let auth = use_context::<Auth>().expect("missing auth context");
	let user = Signal::derive(move ||{
		let id =params.get().get("id").cloned().unwrap_or_default(); 
		CACHE.get(&Uri::full(U::Actor, &id))
	});
	let resource = create_local_resource(
		move || params.get().get("id").cloned().unwrap_or_default(),
		move |id| {
			async move {
				match Http::fetch::<crate::Object>(&format!("{URL_BASE}/actors/{id}/{follow___}/page"), auth).await {
					Err(e) => {
						tracing::error!("failed getting {follow___} for {id}: {e}");
						None
					},
					Ok(x) => {
						Some(x.ordered_items().all_ids())
					},

				}
			}
		}
	);
	view! {
		<div>
			<Breadcrumb back=true >
				actors::view::{follow___}
			</Breadcrumb>
			<div>
				{move || user.get().map(|x| view! { <ActorHeader object=x /> })}
				{move || match resource.get() {
					None => view! { <p>"loading "<span class="dots"></span></p> }.into_view(),
					Some(None) => view! { <code>could not load following</code> }.into_view(),
					Some(Some(arr)) => view! {
						<For
							each=move || arr.clone()
							key=|id| id.clone()
							children=move |id| {
								let actor = match CACHE.get(&id) {
									Some(x) => x,
									None => Arc::new(serde_json::Value::String(id)),
								};
								view! {
									<ActorBanner object=actor />
								}.into_view()
							}
						/ >
					},
				}}
			</div>
		</div>
	}
}
