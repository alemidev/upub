use leptos::*;
use leptos_router::use_params;
use crate::prelude::*;
use super::Timeline;

#[component]
pub fn Feed(
	tl: Timeline,
	#[prop(optional)]
	ignore_filters: bool,
) -> impl IntoView {
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
						<Item item=i sep=true always=ignore_filters />
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

#[component]
pub fn HashtagFeed(tl: Timeline) -> impl IntoView {
	let params = use_params::<IdParam>();
	create_effect(move |_| {
		let current_tag = tl.next.get_untracked()
			.split('/')
			.last()
			.unwrap_or_default()
			.split('?')
			.next()
			.unwrap_or_default()
			.to_string();
		let new_tag = params.get().ok().and_then(|x| x.id).unwrap_or_default();
		if new_tag != current_tag {
			tl.reset(Some(Uri::api(U::Hashtag, &format!("{new_tag}/page"), false)));
		}
	});
	
	view! { <Feed tl=tl /> }
}
