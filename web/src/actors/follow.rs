use leptos::prelude::*;
use leptos_router::hooks::use_params;
use crate::{prelude::*, timeline::any::Loadable};

#[component]
pub fn FollowList(outgoing: bool) -> impl IntoView {
	let follow___ = if outgoing { "following" } else { "followers" };
	let params = use_params::<IdParam>();
	let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
	view! {
		<div class="container">
			<Loadable
				base=format!("{URL_BASE}/actors/{id}/{follow___}/page")
				convert=U::Actor
				element=|obj| view! { <ActorBanner object=obj /><hr/> }
			>
				""
			</Loadable>
		</div>
	}
}

