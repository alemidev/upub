use leptos::prelude::*;
use leptos_router::hooks::use_params;
use crate::prelude::*;

#[component]
pub fn ActorPosts() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<IdParam>();
	Effect::new(move |_| {
		let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
		let tl_url = format!("{}/outbox/page", Uri::api(U::Actor, &id, false));
		if !feeds.user.next.get_untracked().starts_with(&tl_url) {
			feeds.user.reset(Some(tl_url));
		}
	});
	view! {
		<Feed tl=feeds.user />
	}
}

#[component]
pub fn ActorLikes() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<IdParam>();
	Effect::new(move |_| {
		let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
		let likes_url = format!("{}/likes/page", Uri::api(U::Actor, &id, false));
		if !feeds.user.next.get_untracked().starts_with(&likes_url) {
			feeds.user_likes.reset(Some(likes_url));
		}
	});
	view! {
		<Feed tl=feeds.user_likes />
	}
}
