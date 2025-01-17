use leptos::prelude::*;
use leptos_router::hooks::use_params;
use crate::prelude::*;


#[component]
pub fn ObjectReplies() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<IdParam>();
	let id = Signal::derive(move ||
		params.with(|p| p.as_ref().ok().and_then(|x| x.id.as_ref()).cloned()).unwrap_or_default()
	);
	Effect::new(move |_| {
		let tl_url = format!("{}/replies/page", Uri::api(U::Object, &id.get(), false));
		if !feeds.replies.next.get_untracked().starts_with(&tl_url) {
			feeds.replies.reset(Some(tl_url));
		}
	});
	view! {
		<div class="mr-1-r ml-1-r">
			<Thread tl=feeds.replies root=Uri::full(U::Object, &id.get()) />
		</div>
	}
}

#[component]
pub fn ObjectLikes() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<IdParam>();
	let id = Signal::derive(move ||
		params.with(|p| p.as_ref().ok().and_then(|x| x.id.as_ref()).cloned()).unwrap_or_default()
	);
	Effect::new(move |_| {
		let tl_url = format!("{}/likes/page", Uri::api(U::Object, &id.get(), false));
		if !feeds.object_likes.next.get_untracked().starts_with(&tl_url) {
			feeds.object_likes.reset(Some(tl_url));
		}
	});
	view! {
		<div class="mr-1-r ml-1-r">
			<Feed tl=feeds.object_likes ignore_filters=true />
		</div>
	}
}
