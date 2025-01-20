use leptos::prelude::*;
use leptos_router::hooks::use_params;
use crate::prelude::*;

#[component]
pub fn ActorPosts() -> impl IntoView {
	let params = use_params::<IdParam>();
	let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
	view! {
		<Loadable
			base=format!("{}/outbox/page", Uri::api(U::Actor, &id, false))
			element=move |item| view! { <Item item=item sep=true /> }
		/>
	}
}

#[component]
pub fn ActorLikes() -> impl IntoView {
	let params = use_params::<IdParam>();
	let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
	view! {
		<Loadable
			base=format!("{}/likes/page", Uri::api(U::Actor, &id, false))
			element=move |item| view! { <Item item=item sep=true /> }
		/>
	}
}
