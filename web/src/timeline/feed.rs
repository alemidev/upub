use leptos::*;
use crate::prelude::*;
use super::Timeline;

#[component]
pub fn Feed(tl: Timeline) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	if let Some(auto_scroll) = use_context::<Signal<bool>>() {
		let _ = leptos::watch(
			move || auto_scroll.get(),
			move |new, old, _| {
				match old {
					None => tl.spawn_more(auth), // always do it first time
					Some(old) => if *new && new != old {
						tl.spawn_more(auth);
					},
				}
			},
			true,
		);
	}
	view! {
		<div>
			<For
				each=move || tl.feed.get()
				key=|k| k.to_string()
				let:id
			>
				{match CACHE.get(&id) {
					Some(i) => view! {
						<Item item=i sep=true />
					}.into_view(),
					None => view! {
						<p><code>{id}</code>" "[<a href={uri}>go</a>]</p>
						<hr />
					}.into_view(),
				}}
			</For>
		</div>
		{move || if tl.loading.get() { Some(view! { <Loader /> }) } else { None }}
	}
}
