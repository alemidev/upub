use leptos::*;
use crate::prelude::*;
use super::Timeline;

#[component]
pub fn Feed(tl: Timeline) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	if let Some(auto_scroll) = use_context::<Signal<bool>>() {
		let _ = leptos::watch(
			move || auto_scroll.get(),
			move |at_end, _, _| if *at_end { tl.spawn_more(auth) },
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
				{match cache::OBJECTS.get(&id) {
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
