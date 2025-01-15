use ev::MouseEvent;
use leptos::*;
use leptos_router::*;
use crate::{app::FeedRoute, prelude::*, Config};

use apb::Object;

#[component]
pub fn ObjectView() -> impl IntoView {
	let params = use_params_map();
	let matched_route = use_context::<ReadSignal<crate::app::FeedRoute>>().expect("missing route context");
	let auth = use_context::<Auth>().expect("missing auth context");
	let config = use_context::<Signal<Config>>().expect("missing config context");
	let relevant_tl = use_context::<Signal<Option<Timeline>>>().expect("missing relevant timeline context");
	let (loading, set_loading) = create_signal(false);
	let id = Signal::derive(move || params.get().get("id").cloned().unwrap_or_default());
	let object = create_local_resource(
		move || (id.get(), loading.get()),
		move |(oid, _loading)| async move {
			tracing::info!("rerunning fetcher");
			let obj = cache::OBJECTS.resolve(&oid, U::Object, auth).await?;

			// TODO these two can be parallelized
			if let Ok(author) = obj.attributed_to().id() {
				cache::OBJECTS.resolve(&author, U::Actor, auth).await;
			}
			if let Ok(quote) = obj.quote_url().id() {
				cache::OBJECTS.resolve(&quote, U::Object, auth).await;
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

	view! {
		{move || match object.get() {
			None => view! { <Loader /> }.into_view(),
			Some(None) => {
				let raw_id = params.get().get("id").cloned().unwrap_or_default();
				let uid =  uriproxy::uri(URL_BASE, uriproxy::UriClass::Object, &raw_id);
				view! { <p class="center"><code>loading failed</code><sup><small><a class="clean" href={uid} target="_blank">"↗"</a></small></sup></p> }.into_view()
			},
			Some(Some(o)) => {
				tracing::info!("redrawing object");
				view! { <Object object=o.clone() /> }.into_view()
			},
		}}

		<p>
			<span class:tab-active=move || matches!(matched_route.get(), FeedRoute::Context)><a class="clean" href=move || format!("/web/objects/{}", id.get())><span class="emoji ml-2">"🕸️"</span><span class:hidden-on-mobile=move || !matches!(matched_route.get(), FeedRoute::Context)>" context"</span></a></span>
			<span class:tab-active=move || matches!(matched_route.get(), FeedRoute::Replies)><a class="clean" href=move || format!("/web/objects/{}/replies", id.get())><span class="emoji ml-2">"📫"</span><span class:hidden-on-mobile=move || !matches!(matched_route.get(), FeedRoute::Replies)>" replies"</span></a></span>
			<span class:tab-active=move || matches!(matched_route.get(), FeedRoute::ObjectLikes)><a class="clean" href=move || format!("/web/objects/{}/likes", id.get())><span class="emoji ml-2">"⭐"</span><span class:hidden-on-mobile=move || !matches!(matched_route.get(), FeedRoute::ObjectLikes)>" likes"</span></a></span>
			{move || if auth.present() {
				if loading.get() {
					Some(view! {
						<span style="float: right">
							"fetching "<span class="dots"></span>
						</span>
					})
				} else {
					Some(view! {
						<span style="float: right">
							<a
								class="clean"
								on:click=move |ev| fetch_cb(ev, set_loading, id.get(), auth, config, relevant_tl)
								href="#"
							>
								<span class="emoji ml-2">"↺ "</span>"fetch"
							</a>
						</span>
					})
				}
			} else {
				None
			}}
		</p>
		<hr class="color" />

		{move || if object.get().is_some() {
			tracing::info!("redrawing outlet");
			Some(view! { <Outlet /> })
		} else {
			None
		}}
	}
}

fn fetch_cb(ev: MouseEvent, set_loading: WriteSignal<bool>, oid: String, auth: Auth, config: Signal<Config>, relevant_tl: Signal<Option<Timeline>>) {
	let api = Uri::api(U::Object, &oid, false);
	ev.prevent_default();
	set_loading.set(true);
	spawn_local(async move {
		if let Err(e) = Http::fetch::<serde_json::Value>(&format!("{api}/replies?fetch=true"), auth).await {
			tracing::error!("failed crawling replies for {oid}: {e}");
		}
		cache::OBJECTS.invalidate(&Uri::full(U::Object, &oid));
		tracing::info!("invalidated {}", Uri::full(U::Object, &oid));
		set_loading.set(false);
		relevant_tl.get().inspect(|x| x.refresh(auth, config));
	});
}
