use leptos::prelude::*;
use leptos_router::hooks::use_params;
use crate::prelude::*;

#[component]
pub fn CommunitiesList() -> impl IntoView {
	let params = use_params::<IdParam>();
	let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
	view! {
		<div class="container">
			<Loadable
				base=format!("{URL_BASE}/actors/{id}/communities/page")
				convert=U::Actor
				element=|obj| view! { <ActorBanner object=obj /><hr/> }
			/>
		</div>
	}
}

