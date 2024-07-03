use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use std::sync::Arc;

use apb::Collection;

#[component]
pub fn FollowList(outgoing: bool) -> impl IntoView {
	let follow___ = if outgoing { "following" } else { "followers" };
	let symbol = if outgoing { "👥" } else { "📢" };
	let params = use_params::<super::IdParam>();
	let auth = use_context::<Auth>().expect("missing auth context");
	let resource = create_local_resource(
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
		<code class="cw center color mt-1 mr-3 ml-3"><span class="emoji">{symbol}</span>" "<b>{follow___}</b></code>
		<div class="tl ml-3-r mr-3-r pl-1 pt-1 pb-1">
			{move || match resource.get() {
				None => view! { <Loader /> }.into_view(),
				Some(Err(e)) => view! { <p class="center">could not load {follow___}: {e}</p> }.into_view(),
				Some(Ok(arr)) => view! {
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
							}.into_view()
						}
					/ >
				}.into_view(),
			}}
		</div>
	}
}

