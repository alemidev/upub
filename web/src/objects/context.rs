use apb::Object;
use leptos::*;
use leptos_router::*;
use crate::prelude::*;


#[component]
pub fn ObjectContext() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<IdParam>();
	let id = move || {
		let id = params.with(|p| p.as_ref().ok().and_then(|x| x.id.as_ref()).cloned()).unwrap_or_default();
		Uri::full(U::Object, &id)
	};
	let context_id = Signal::derive(move ||
		cache::OBJECTS.get(&id())
			.and_then(|x| x.context().id().ok())
			.unwrap_or_default()
	);
	create_effect(move |_| {
		let tl_url = format!("{}/context/page", Uri::api(U::Object, &context_id.get(), false));
		if !feeds.context.next.get_untracked().starts_with(&tl_url) {
			feeds.context.reset(Some(tl_url));
		}
	});
	view! {
		<div class="mr-1-r ml-1-r">
			<Thread tl=feeds.context root=id() />
		</div>
	}
}
