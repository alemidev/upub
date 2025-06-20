use leptos::{either::Either, ev::MouseEvent};
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_params_map;
use crate::{app::FeedRoute, prelude::*};

use apb::{ActivityMut, Base, Object};

#[component]
pub fn ListView() -> impl IntoView {
	let params = use_params_map();
	let matched_route = use_context::<ReadSignal<crate::app::FeedRoute>>().expect("missing route context");
	let auth = use_context::<Auth>().expect("missing auth context");
	// TODO do we really need this loading signal?
	let (loading, set_loading) = signal(false);
	let id = Signal::derive(move || params.get().get("id").unwrap_or_default());
	let target_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let list = LocalResource::new(
		move || {
			let (lid, _loading) = (id.get(), loading.get());
			async move {
				tracing::info!("rerunning fetcher");
				let obj = cache::OBJECTS.fetch(&lid, U::List, auth).await?;

				// TODO these two can be parallelized
				if let Ok(author) = obj.attributed_to().id() {
					cache::OBJECTS.fetch(&author, U::Actor, auth).await;
				}

				Some(obj)
			}
		}
	);

	view! {
		{move || match list.try_get() {
			None => ().into_any(), // already disposed
			Some(None) => view! { <Loader /> }.into_any(), // still loading
			Some(Some(None)) => { // error loading
				let raw_id = params.get().get("id").unwrap_or_default();
				let uid =  uriproxy::uri(URL_BASE, uriproxy::UriClass::Object, &raw_id);
				view! { <p class="center"><code>loading failed</code><sup><small><a class="clean" href={uid} target="_blank">"↗"</a></small></sup></p> }.into_any()
			},
			Some(Some(Some(o))) => { // loaded ok
				tracing::info!("redrawing list");
				view! {
					<List list=o.clone() />
					<blockquote class="mt-1">
						<details class="cw">
							<summary>
								<code class="cw center color">add to list</code>
							</summary>
								<table class="align w-100">
									<tr>
										<td class="w-66"><input class="w-100" type="text" node_ref=target_ref placeholder="id" /></td>
										<td class="w-33">
											<input class="w-100" type="submit" value="add" on:click=move |ev| {
												ev.prevent_default();
												let object_id = target_ref.get().map(|x| x.value()).filter(|x| !x.is_empty());
												let target_id = o.id();
												if let (Some(oid), Ok(tid)) = (object_id, target_id) {
													let payload = apb::new()
														.set_activity_type(Some(apb::ActivityType::Add))
														.set_target(apb::Node::link(tid))
														.set_object(apb::Node::link(oid));
													leptos::task::spawn_local(async move {
														match crate::Http::post(&auth.outbox(), &payload, auth).await {
															Ok(()) => {
																if let Some(target_ref_element) = target_ref.get() {
																	target_ref_element.set_value("");
																}
															},
															Err(e) => { tracing::error!("{e}"); },
														}
													});
												} else {
													tracing::error!("missing object_id or target_id");
												}
											} />
										</td>
									</tr>
								</table>
						</details>
					</blockquote>

				}.into_any()
			},
		}}

		<p>
			<span class:tab-active=move || matches!(matched_route.get(), FeedRoute::ListMembers)><a class="clean" href=move || format!("/web/lists/{}", id.get())><span class="emoji ml-2">"👥 "</span><span class:hidden-on-mobile=move || !matches!(matched_route.get(), FeedRoute::ListMembers)>" members"</span></a></span>
			<span class:tab-active=move || matches!(matched_route.get(), FeedRoute::ListFeed)><a class="clean" href=move || format!("/web/lists/{}/feed", id.get())><span class="emoji ml-2">"📫 "</span><span class:hidden-on-mobile=move || !matches!(matched_route.get(), FeedRoute::ListFeed)>" feed"</span></a></span>
		</p>
		<hr class="color" />

		{move || if list.try_get().is_some_and(|x| x.is_some()) {
			tracing::info!("redrawing outlet");
			Some(view! { <Outlet /> })
		} else {
			None
		}}
	}
}
