use leptos::{either::Either, prelude::*};
use leptos_router::hooks::use_params;
use crate::prelude::*;
use super::Timeline;

#[component]
pub fn Feed(
	tl: Timeline,
	#[prop(optional)]
	ignore_filters: bool,
) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	if let Some(auto_scroll) = use_context::<Signal<bool>>() {
		let _ = Effect::watch(
			move || auto_scroll.get(),
			move |at_end, _, _| if *at_end { tl.spawn_more(auth, config) },
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
					Some(i) => Either::Left(view! {
						<Item item=i sep=true always=ignore_filters />
					}),
					None => Either::Right(view! {
						<p><code>{id}</code>" "[<a href={uri}>go</a>]</p>
						<hr />
					}),
				}}
			</For>
		</div>
		{move || if tl.loading.get() { Some(view! { <Loader /> }) } else { None }}
	}
}

#[component]
pub fn HashtagFeed(tl: Timeline) -> impl IntoView {
	let params = use_params::<IdParam>();
	Effect::new(move |_| {
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
