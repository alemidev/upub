use leptos::{either::Either, prelude::*};
use leptos_router::hooks::query_signal;
use crate::prelude::*;

#[component]
pub fn ThreadsPage() -> impl IntoView {
	let (days, set_days) = query_signal::<i32>("days");
	let (skip, set_skip) = query_signal::<i32>("skip");
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		{move || if auth.present() {
			Either::Left(view! {
				<blockquote class="mb-3">
					<details class="cw">
						<summary>
							<code class="cw center color ml-s w-100">subscriptions</code>
						</summary>
						<div class="ml-2 mt-1 mb-1">
							<Loadable
								base=format!("{URL_BASE}/actors/{}/groups/page", auth.username())
								convert=U::Actor
								element=|obj| view! { <ActorBanner object=obj /><hr /> }
							/>
						</div>
					</details>
				</blockquote>
			})
		} else {
			Either::Right(())
		}}
		<div class="pl-1 pr-1">
			<table class="w-100 fixed">
				<tr>
					<td class="pa-1">
						"in last "
						<input type="number" size="4" placeholder="1"
							prop:value=move || days.get()
							on:input=move |ev| {
								ev.prevent_default();
								let days = event_target_value(&ev).parse().ok();
								set_days.set(days);
						} />
						" days"
					</td>
					<td class="pa-1">
						"skip "
						<input type="number" size="4" placeholder="0"
							prop:value=move || skip.get()
							on:input=move |ev| {
								ev.prevent_default();
								let skip = event_target_value(&ev).parse().ok();
								set_skip.set(skip);
						} />
						" most recent days"
					</td>
				</tr>
			</table>
		</div>
		{move || {
			let skip_n = skip.get().unwrap_or(0);
			let days_n = days.get().unwrap_or(1);
			view! {
				<div class="mt-3">
					<Loadable
						base=format!("{URL_BASE}/threads/page?days={}&skip={skip_n}", days_n + skip_n)
						element=move |obj| view! { <Item item=obj sep=true /> }
					/>
				</div>
			}
		}}
	}
}
