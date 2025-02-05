use leptos::prelude::*;
use crate::prelude::*;

#[component]
pub fn Breadcrumb(
	#[prop(optional)]
	back: bool,
	children: Children,
) -> impl IntoView {
	view! {
		<div class="tl-header w-100 center" >
			{if back { Some(view! {
				<a class="breadcrumb mr-1" href="javascript:history.back()" ><b>"<<"</b></a>
			})} else { None }}
			<b>{crate::NAME}</b>" :: "{children()}
		</div>
	}
}

#[component]
pub fn Navigator(notifications: ReadSignal<u64>) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let (query, set_query) = signal("".to_string());
	view! {
		<form action={move|| format!("/web/search?q={}", query.get())}>
			<table class="align">
				<tr>
					<td class="w-100">
						<input type="text" placeholder="search" class="w-100" on:input=move |ev| {
							set_query.set(event_target_value(&ev))
						} />
					</td>
					<td>
						<a href={move|| format!("/web/search?q={}", query.get())}><input type="submit" value="go" /></a>
					</td>
				</tr>
			</table>
		</form>
		<table class="align w-100">
			<tr><td colspan="2"><a href="/web/home"><input class="w-100" type="submit" class:hidden=move || !auth.present() value="home timeline" /></a></td></tr>
			<tr><td colspan="2"><a href="/web/global"><input class="w-100" type="submit" value="global timeline" /></a></td></tr>
			<tr><td colspan="2"><a href="/web/local"><input class="w-100" type="submit" value="local timeline" /></a></td></tr>
			<tr><td colspan="2"><a href="/web/notifications"><input class="w-100" type="submit" class:hidden=move || !auth.present() value=move || format!("notifications [{}]", notifications.get()) /></a></td></tr>
			<tr>
				<td class="w-50"><a href="/web/about"><input class="w-100" type="submit" value="about" /></a></td>
				<td class="w-50"><a href="/web/config"><input class="w-100" type="submit" value="config" /></a></td>
			</tr>
			// <tr><td colspan="2"><a href="/web/groups"><input class="w-100" type="submit" value="groups" /></a></td></tr> // still too crude, don't include in navigation
			<tr><td colspan="2"><a href="/web/explore"><input class="w-100" type="submit" class:hidden=move || !auth.present() value="explore" /></a></td></tr>
		</table>
	}
}
