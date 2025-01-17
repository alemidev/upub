use leptos::prelude::*;
use leptos_router::hooks::use_params;
use crate::prelude::*;

use std::sync::Arc;

use apb::Collection;

#[component]
pub fn FollowList(outgoing: bool) -> impl IntoView {
	let follow___ = if outgoing { "following" } else { "followers" };
	let params = use_params::<IdParam>();
	let auth = use_context::<Auth>().expect("missing auth context");
	// TODO this was a LocalResource!
	let resource = Resource::new(
		move || params.get().ok().and_then(|x| x.id).unwrap_or_default(),
		move |id| {
			async move {
				Ok::<_, String>(
					Http::fetch::<serde_json::Value>(&format!("{URL_BASE}/actors/{id}/{follow___}/page"), auth)
						.await
						.map_err(|e| e.to_string())?
						.ordered_items()
						.all_ids()
				)
			}
		}
	);
	view! {
		<div class="tl ml-3-r mr-3-r pl-1 pt-1 pb-1">
			{move || match resource.get() {
				None => view! { <Loader /> }.into_any(),
				Some(Err(e)) => {
					tracing::error!("could not load followers: {e}");
					view! { <code class="cw center color">{follow___}" unavailable"</code> }.into_any()
				},
				Some(Ok(mut arr)) => {
					// TODO cheap fix: server gives us follows from oldest to newest
					//      but it's way more convenient to have them other way around
					//      so we reverse them just after loading them
					arr.reverse();
					view! {
						<For
							each=move || arr.clone()
							key=|id| id.clone()
							children=move |id| {
								let actor = match cache::OBJECTS.get(&id) {
									Some(x) => x,
									None => Arc::new(serde_json::Value::String(id)),
								};
								view! {
									<ActorBanner object=actor />
									<hr />
								}.into_any()
							}
						/ >
					}.into_any()
				},
			}}
		</div>
	}
}

