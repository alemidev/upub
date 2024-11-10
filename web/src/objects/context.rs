use leptos::*;
use leptos_router::*;
use crate::prelude::*;


#[component]
pub fn ObjectContext() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<IdParam>();
	let id = Signal::derive(move || {
		params.get().ok().and_then(|x| x.id).unwrap_or_default()
	});
	create_effect(move |_| {
		let tl_url = format!("{}/context/page", Uri::api(U::Object, &id.get(), false));
		if !feeds.context.next.get_untracked().starts_with(&tl_url) {
			feeds.context.reset(Some(tl_url));
		}
	});
	view! {
		<div class="mr-1-r ml-1-r">
			<Thread tl=feeds.context root=Uri::full(U::Object, &id.get()) />
		</div>
	}
}
