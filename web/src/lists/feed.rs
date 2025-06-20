use leptos::prelude::*;
use leptos_router::hooks::use_params;
use crate::prelude::*;

#[component]
pub fn ListFeed() -> impl IntoView {
	let params = use_params::<IdParam>();
	// TODO do it reactively
	let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
	view! {
		<Loadable
			base=format!("{}/feed/page", Uri::api(U::List, &id, false))
			element=move |item| view! { <Item item=item sep=true /> }
		/>
	}
}
