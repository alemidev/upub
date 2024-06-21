use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn ActorActivity() -> impl IntoView {
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let params = use_params::<super::IdParam>();
	let id = Signal::derive(move || {
		let id = params.get_untracked().ok().and_then(|x| x.id).unwrap_or_default();
		let tl_url = format!("{}/outbox/page", Uri::api(U::Actor, &id, false));
		if !feeds.user.next.get_untracked().starts_with(&tl_url) {
			feeds.user.reset(Some(tl_url));
		}
		id
	});
	view! {
		<code class="cw color center mt-1 mb-1 ml-3 mr-3">
			<a class="clean" href={format!("/web/actors/{}", id.get())}><span class="emoji">"🖂"</span>" posts"</a>
			" | "
			<b>activity</b>" "<span class="emoji">"@"</span>
		</code>
		<Feed tl=feeds.user />
	}
}
