use leptos::*;
use crate::prelude::*;

#[component]
pub fn TimelinePage(name: &'static str, tl: Timeline) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<div>
			<Breadcrumb back=false>
				{name}
				<a class="clean ml-1" href="#" on:click=move |_| {
					tl.reset(tl.next.get().split('?').next().unwrap_or_default().to_string());
					spawn_local(async move {
						if let Err(e) = tl.more(auth).await {
							tracing::error!("error fetching more items for timeline: {e}");
						}
					})
				}><span class="emoji">
					"\u{1f5d8}"
				</span></a>
			</Breadcrumb>
			<div class="mt-s mb-s" >
				<TimelineFeed tl=tl />
			</div>
		</div>
	}
}
