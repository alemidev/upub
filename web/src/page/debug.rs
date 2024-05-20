use std::sync::Arc;

use leptos::*;
use crate::prelude::*;

#[component]
pub fn DebugPage() -> impl IntoView {
	let (object, set_object) = create_signal(Arc::new(serde_json::Value::String(
		"use this view to fetch remote AP objects and inspect their content".into())
	));
	let cached_ref: NodeRef<html::Input> = create_node_ref();
	let auth = use_context::<Auth>().expect("missing auth context");
	let (query, set_query) = create_signal("".to_string());
	view! {
		<div>
			<Breadcrumb back=true>config :: devtools</Breadcrumb>
			<div class="mt-1" >
				<form on:submit=move|ev| {
					ev.prevent_default();
					let cached = cached_ref.get().map(|x| x.checked()).unwrap_or_default();
					let fetch_url = query.get();
					if cached {
						match CACHE.get(&fetch_url) {
							Some(x) => set_object.set(x),
							None => set_object.set(Arc::new(serde_json::Value::String("not in cache!".into()))),
						}
					} else {
						let url = format!("{URL_BASE}/proxy?id={fetch_url}");
						spawn_local(async move { set_object.set(Arc::new(debug_fetch(&url, auth).await)) });
					}
				} >
				<table class="align w-100" >
					<tr>
						<td>
							<small><a
								href={move|| Uri::web(U::Object, &query.get())}
							>obj</a>
								" "
							<a
								href={move|| Uri::web(U::User, &query.get())}
							>usr</a></small>
						</td>
						<td class="w-100"><input class="w-100" type="text" on:input=move|ev| set_query.set(event_target_value(&ev)) placeholder="AP id" /></td>
						<td><input type="submit" class="w-100" value="fetch" /></td>
						<td><input type="checkbox" title="cached" value="cached" node_ref=cached_ref /></td>
					</tr>
				</table>
				</form>
			</div>
			<pre class="ma-1" >
				{move || serde_json::to_string_pretty(object.get().as_ref()).unwrap_or("unserializable".to_string())}
			</pre>
		</div>
	}
}

// this is a rather weird way to fetch but i want to see the bare error text if it fails!
async fn debug_fetch(url: &str, token: Auth) -> serde_json::Value {
	match Http::request::<()>(reqwest::Method::GET, url, None, token).await {
		Err(e) => serde_json::Value::String(format!("[!] failed sending request: {e}")),
		Ok(res) => match res.text().await {
			Err(e) => serde_json::Value::String(format!("[!] invalid response body: {e}")),
			Ok(x) => match serde_json::from_str(&x) {
				Err(_) => serde_json::Value::String(x),
				Ok(v) => v,
			},
		}
	}
}
