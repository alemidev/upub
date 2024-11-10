use leptos::*;
use leptos_router::*;
use crate::prelude::*;


#[component]
pub fn ObjectReplies() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<IdParam>();
	create_effect(move |_| {
		let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
		let tl_url = format!("{}/replies/page", Uri::api(U::Object, &id, false));
		if !feeds.replies.next.get_untracked().starts_with(&tl_url) {
			feeds.replies.reset(Some(tl_url));
		}
	});
	view! {
		<div class="mr-1-r ml-1-r">
			<Thread tl=feeds.replies root=params.get().unwrap().id.unwrap() />
		</div>
	}
}
