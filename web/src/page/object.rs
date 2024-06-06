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
	let object = create_local_resource(
		move || params.get().get("id").cloned().unwrap_or_default(),
		move |oid| async move {
			let obj = match CACHE.get(&Uri::full(U::Object, &oid)) {
				Some(x) => x.clone(),
				None => {
					let obj = match Http::fetch::<serde_json::Value>(&Uri::api(U::Object, &oid, true), auth).await {
						Ok(obj) => Arc::new(obj),
						Err(e) => {
							tracing::error!("failed loading object from backend: {e}");
							return None;
						},
					};
					if let Ok(author) = obj.attributed_to().id() {
						if let Ok(user) = Http::fetch::<serde_json::Value>(
							&Uri::api(U::Actor, author, true), auth
						).await {
							CACHE.put(Uri::full(U::Actor, author), Arc::new(user));
						}
					}
					CACHE.put(Uri::full(U::Object, &oid), obj.clone());
					obj
				}
			};
			if let Ok(ctx) = obj.context().id() {
				let tl_url = format!("{}/context/page", Uri::api(U::Object, ctx, true));
				if !tl.next.get().starts_with(&tl_url) {
					tl.reset(tl_url);
				}
			}

			Some(obj)
		}
	);
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
