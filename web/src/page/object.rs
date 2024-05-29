use std::sync::Arc;

use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use apb::{Base, Object};

#[component]
pub fn ObjectPage(tl: Timeline) -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Auth>().expect("missing auth context");
	let id = params.get().get("id").cloned().unwrap_or_default();
	let uid =  uriproxy::uri(URL_BASE, uriproxy::UriClass::Object, &id);
	let object = create_local_resource(move || params.get().get("id").cloned().unwrap_or_default(), move |oid| {
		async move {
			match CACHE.get(&Uri::full(U::Object, &oid)) {
				Some(x) => Some(x.clone()),
				None => {
					let obj = Http::fetch::<serde_json::Value>(&Uri::api(U::Object, &oid, true), auth).await.ok()?;
					let obj = Arc::new(obj);
					if let Some(author) = obj.attributed_to().id() {
						if let Ok(user) = Http::fetch::<serde_json::Value>(
							&Uri::api(U::Actor, &author, true), auth
						).await {
							CACHE.put(Uri::full(U::Actor, &author), Arc::new(user));
						}
					}
					CACHE.put(Uri::full(U::Object, &oid), obj.clone());
					Some(obj)
				}
			}
		}
	});
	view! {
		<div>
			<Breadcrumb back=true >
				objects::view
				<a
					class="clean ml-1" href="#"
					class:hidden=move || tl.is_empty()
					on:click=move |_| {
						tl.reset(tl.next.get().split('?').next().unwrap_or_default().to_string());
						tl.more(auth);
				}><span class="emoji">
					"\u{1f5d8}"
				</span></a>
			</Breadcrumb>
			<div class="ma-2" >
				{move || match object.get() {
					None => view! { <p class="center"> loading ... </p> }.into_view(),
					Some(None) => {
						let uid = uid.clone();
						view! { <p class="center"><code>loading failed</code><sup><small><a class="clean" href={uid} target="_blank">"↗"</a></small></sup></p> }.into_view()
					},
					Some(Some(o)) => {
						let object = o.clone();
						let tl_url = format!("{}/page", Uri::api(U::Context, &o.context().id().unwrap_or_default(), false));
						if !tl.next.get().starts_with(&tl_url) {
							tl.reset(tl_url);
						}
						view!{
							<Object object=object />
							<div class="ml-1 mr-1 mt-2">
								<TimelineReplies tl=tl root=o.id().unwrap_or_default().to_string() />
							</div>
						}.into_view()
					},
				}}
			</div>
		</div>
	}
}
