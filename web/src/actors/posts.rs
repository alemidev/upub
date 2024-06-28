use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn ActorPosts() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<super::IdParam>();
	Signal::derive(move || {
		let id = params.get_untracked().ok().and_then(|x| x.id).unwrap_or_default();
		let tl_url = format!("{}/outbox/page", Uri::api(U::Actor, &id, false));
		if !feeds.user.next.get_untracked().starts_with(&tl_url) {
			feeds.user.reset(Some(tl_url));
		}
		id
	}).track();
	view! {
		<code class="cw color center mt-1 mb-1 ml-3 mr-3">
			<span class="emoji">"🖂"</span>" "<b>posts</b>
		</code>
		<Feed tl=feeds.user />
	}
}
