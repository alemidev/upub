use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use apb::{Base, Object};

#[component]
pub fn ObjectView() -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Auth>().expect("missing auth context");
	let object = create_local_resource(
		move || params.get().get("id").cloned().unwrap_or_default(),
		move |oid| async move {
			let obj = cache::OBJECTS.resolve(&oid, U::Object, auth).await?;
			if let Ok(author) = obj.attributed_to().id() {
				cache::OBJECTS.resolve(author, U::Actor, auth).await;
			}
			Some(obj)

			// if let Ok(ctx) = obj.context().id() {
			// 	let tl_url = format!("{}/context/page", Uri::api(U::Object, ctx, false));
			// 	if !feeds.context.next.get_untracked().starts_with(&tl_url) {
			// 		feeds.context.reset(Some(tl_url));
			// 	}
			// }
		}
	);

	{move || match object.get() {
		None => view! { <Loader /> }.into_view(),
		Some(None) => {
			let raw_id = params.get().get("id").cloned().unwrap_or_default();
			let uid =  uriproxy::uri(URL_BASE, uriproxy::UriClass::Object, &raw_id);
			view! { <p class="center"><code>loading failed</code><sup><small><a class="clean" href={uid} target="_blank">"↗"</a></small></sup></p> }.into_view()
		},
		Some(Some(o)) => {
			let object = o.clone();
			let base = Uri::web(U::Object, o.id().unwrap_or_default());
			view!{
				<Object object=object />
				<hr class="color ma-2" />
				<code class="cw color center mt-1 mb-1 ml-3 mr-3">
					<a class="clean" href=format!("{base}/context")><span class="emoji">"🕸️"</span>" "<b>context</b></a>" | "<a class="clean" href=format!("{base}/replies")><span class="emoji">"📫"</span>" "<b>replies</b></a>
					{if auth.present() {
						Some(view! {
							" | "<a class="clean" href="#crawl" on:click=move |_| crawl(base.clone(), auth) ><span class="emoji">"↺"</span></a>
						})
					} else { None }}
				</code>
				<Outlet />
			}.into_view()
		},
	}}
}

fn crawl(base: String, auth: Auth) {
	spawn_local(async move {
		if let Err(e) = Http::fetch::<serde_json::Value>(&format!("{URL_BASE}{base}/replies?fetch=true"), auth).await {
			tracing::error!("failed crawling replies for {base}: {e}");
		}
	});
}
