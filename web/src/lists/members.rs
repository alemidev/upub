use apb::Object;
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::use_params;
use crate::prelude::*;

#[component]
pub fn ListMembers() -> impl IntoView {
	let params = use_params::<IdParam>();
	// TODO do it reactively
	let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
	view! {
		<Loadable
			base=format!("{}/page", Uri::api(U::List, &id, false))
			element=move |item| match item.object_type() {
				Ok(apb::ObjectType::Actor(_)) => Either::Left(view! { <ActorBanner object=item /> }),
				_ => Either::Right(view! { <Item item=item sep=true /> }),
			}
		/>
	}
}
