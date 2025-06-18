use leptos::{either::Either, prelude::*};
use crate::prelude::*;

#[component]
pub fn ThreadsPage() -> impl IntoView {
	let (days, set_days) = signal(Some(30));
	let (skip, set_skip) = signal(Some(0));
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
						<input type="number" size="4" placeholder="30"
							prop:value=move || days.get()
							on:input=move |ev| {
								ev.prevent_default();
								set_days.set(event_target_value(&ev).parse().ok());
						} />
						" days"
					</td>
					<td class="pa-1">
						"skip "
						<input type="number" size="4" placeholder="0"
							prop:value=move || skip.get().unwrap_or(0)
							on:input=move |ev| {
								ev.prevent_default();
								set_skip.set(event_target_value(&ev).parse().ok());
						} />
						" most recent days"
					</td>
				</tr>
			</table>
		</div>
		{move || match (days.get(), skip.get()) {
			(Some(days_n), Some(skip_n)) => Either::Right(view! {
				<div class="mt-3">
					<Loadable
						base=format!("{URL_BASE}/threads/page?days={days_n}&skip={skip_n}")
						element=move |obj| view! { <Item item=obj sep=true /> }
					/>
				</div>
			}),
			_ => Either::Left(view! {
				<p class="center"><code>invalid days/skip parameters</code></p>
			}),
		}}
	}
}
