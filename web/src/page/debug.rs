use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn DebugPage() -> impl IntoView {
	let query_params = use_query_map();
	let auth = use_context::<Auth>().expect("missing auth context");
	let (cached, set_cached) = create_signal(false);
	let (error, set_error) = create_signal(false);
	let (plain, set_plain) = create_signal(false);
	let (text, set_text) = create_signal("".to_string());
	let navigate = use_navigate();

	let cached_query = move || (
		query_params.with(|params| params.get("q").cloned().unwrap_or_default()),
		cached.get(),
	);
	let object = create_local_resource(
		cached_query,
		move |(query, cached)| async move {
			set_text.set(query.clone());
			set_error.set(false);
			if query.is_empty() { return serde_json::Value::Null };
			if cached {
				match cache::OBJECTS.get(&query) {
					Some(x) => (*x).clone(),
					None => {
						set_error.set(true);
						serde_json::Value::Null
					},
				}
			} else {
				debug_fetch(&format!("{URL_BASE}/fetch?uri={query}"), auth, set_error).await
			}
		}
	);
	let loading = object.loading();


	view! {
		<div>
			<div class="mt-1" >
				<form on:submit=move|ev| {
					ev.prevent_default();
					navigate(&format!("/web/dev?q={}", text.get()), NavigateOptions::default());
				} >
					<table class="align w-100">
						<tr>
							<td class="w-100">
								<input class="w-100" type="text"
									prop:value=text
									on:input=move|ev| set_text.set(event_target_value(&ev))
									placeholder="AP id"
								/>
							</td>
							<td>
								<input type="submit" class="w-100" value="fetch" />
							</td>
							<td>
								<input type="checkbox" title="load from local cache" value="cached"
									class:spinner=loading
									prop:checked=cached
									on:input=move |ev| set_cached.set(event_target_checked(&ev))
								/>
							</td>
						</tr>
					</table>
				</form>
			</div>
			<pre class="ma-1" class:striped=error>
				{move || match object.get() {
					None => view! { <p class="center"><span class="dots"></span></p> }.into_view(),
					Some(o) => if plain.get() {
						serde_json::to_string_pretty(&o).unwrap_or_else(|e| e.to_string()).into_view()
					} else {
						view! { <DocumentNode obj=o /> }.into_view()
					},
				}}
			</pre>
			<p class="center">
					<input type="checkbox" title="show plain (and valid) json" value="plain" prop:checked=plain on:input=move |ev| set_plain.set(event_target_checked(&ev)) />
					" raw :: "
					<a href={move|| Uri::web(U::Object, &text.get())} >obj</a>
					" :: "
					<a href={move|| Uri::web(U::Actor, &text.get())} >usr</a>
					" :: "
					<a href=move || cached_query().0 target="_blank" rel="nofollow noreferrer">ext</a>
					" :: "
					<a href="#"
						onclick={move ||
							format!(
								"javascript:navigator.clipboard.writeText(`{}`)",
								object.get().map(|x| serde_json::to_string(&x).unwrap_or_default()).unwrap_or_default()
							)
					} >copy</a>
			</p>
		</div>
	}
}

// this is a rather weird way to fetch but i want to see the bare error text if it fails!
async fn debug_fetch(url: &str, token: Auth, error: WriteSignal<bool>) -> serde_json::Value {
	match Http::request::<()>(reqwest::Method::GET, url, None, token).await {
		Ok(res) => {
			if res.error_for_status_ref().is_err() {
				error.set(true); // this is an error but body could still be useful json
			}
			match res.text().await {
				Ok(x) => match serde_json::from_str(&x) {
					Ok(v) => v,
					Err(_) => {
						error.set(true);
						serde_json::Value::String(x)
					},
				},
				Err(e) => {
					error.set(true);
					serde_json::Value::String(format!("[!] invalid response body: {e}"))
				},
			}
		},
		Err(e) => {
			error.set(true);
			serde_json::Value::String(format!("[!] failed sending request: {e}"))
		},
	}
}

#[component]
fn DocumentNode(obj: serde_json::Value, #[prop(optional)] depth: usize) -> impl IntoView {
	let prefix = "  ".repeat(depth);
	let newline_replace = format!("\n{prefix}  ");
	match obj {
		serde_json::Value::Null => view! { <b>null</b> }.into_view(),
		serde_json::Value::Bool(x) => view! { <b>{x}</b> }.into_view(),
		serde_json::Value::Number(n) => view! { <b>{n.to_string()}</b> }.into_view(),
		serde_json::Value::String(s) => {
			if s.starts_with("https://") || s.starts_with("http://") {
				view! {
					<a href=format!("/web/dev?q={s}")>{s}</a>
				}.into_view()
			} else {
				let pretty_string = s
					.replace("<br/>", "<br/>\n")
					.replace("<br>", "<br>\n")
					.replace('\n', &newline_replace);
				view! {
					"\""<span class="json-text"><i>{pretty_string}</i></span>"\""
				}.into_view()
			}
		},
		serde_json::Value::Array(arr) => if arr.is_empty() { 
			view! { "[]" }.into_view()
		} else {
			view! {
				"[\n"
					{arr.into_iter().map(|x| view! {
						{prefix.clone()}"  "<DocumentNode obj=x depth=depth+1 />"\n"
					}).collect_view()}
				{prefix.clone()}"]"
			}.into_view()
		},
		serde_json::Value::Object(map) => if map.is_empty() {
			view! { "{}" }.into_view()
		} else {
			view! {
				"{\n"
					{
						map.into_iter()
							.map(|(k, v)| view! {
								{prefix.clone()}"  "<span class="json-key"><b>{k}</b></span>": "<DocumentNode obj=v depth=depth+1 />"\n"
							})
							.collect_view()
					}
				{prefix.clone()}"}"
			}.into_view()
		},
	}
}
