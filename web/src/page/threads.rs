use leptos::{either::Either, prelude::*};
use leptos_router::{hooks::{use_navigate, use_query_map}, NavigateOptions};
use crate::prelude::*;

#[component]
pub fn ThreadsPage() -> impl IntoView {
	let (days, set_days) = signal(Some(1));
	let (skip, set_skip) = signal(Some(0));
	let auth = use_context::<Auth>().expect("missing auth context");
	use_query_map().with(|q| {
		if let Some(d) = q.get("days") {
			if let Ok(days_number) = d.parse() {
				set_days.set(Some(days_number));
			}
		}

		if let Some(s) = q.get("skip") {
			if let Ok(skip_number) = s.parse() {
				set_skip.set(Some(skip_number));
			}
		}
	});
	let navigate = use_navigate();
	let _navigate = navigate.clone();
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
								let days = event_target_value(&ev).parse().ok();
								set_days.set(days);
								navigate(&threads_query_params(days, skip.get()), threads_nav_options());
						} />
						" days"
					</td>
					<td class="pa-1">
						"skip "
						<input type="number" size="4" placeholder="0"
							prop:value=move || skip.get().unwrap_or(0)
							on:input=move |ev| {
								ev.prevent_default();
								let skip = event_target_value(&ev).parse().ok();
								set_skip.set(skip);
								_navigate(&threads_query_params(days.get(), skip), threads_nav_options());
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
						base=format!("{URL_BASE}/threads/page?days={}&skip={skip_n}", days_n + skip_n)
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

fn threads_query_params(days: Option<i32>, skip: Option<i32>) -> String {
	let mut query_string = "/web/threads".to_string();
	if let Some(days_n) = days {
		query_string.push('?');
		query_string.push_str("days=");
		query_string.push_str(&days_n.to_string());
	}

	if let Some(skip_n) = skip {
		query_string.push(if query_string.contains('?') { '&' } else { '?' });
		query_string.push_str("skip=");
		query_string.push_str(&skip_n.to_string());
	}

	query_string
}

#[inline]
fn threads_nav_options() -> NavigateOptions {
	NavigateOptions { resolve: false, replace: true, scroll: false, state: leptos_router::location::State::new(None) }
}
