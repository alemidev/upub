use apb::{Actor, Base, Collection, Object};

use leptos::*;
use leptos_router::*;
use crate::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
	view! {
		<div>
			<Breadcrumb>about</Breadcrumb>
			<div class="mt-s mb-s" >
				<p><code>μpub</code>" is a micro social network powered by "<a href="">ActivityPub</a></p>
				<p><i>"the "<a href="https://en.wikipedia.org/wiki/Fediverse">fediverse</a>" is an ensemble of social networks, which, while independently hosted, can communicate with each other"</i></p>
				<p>content is aggregated in timelines, logged out users can only access global server timeline</p>
			</div>
		</div>
	}
}

#[component]
pub fn ConfigPage() -> impl IntoView {
	view! {
		<div>
			<Breadcrumb>config</Breadcrumb>
			<div class="mt-s mb-s" >
				<p><code>"not implemented :("</code></p>
			</div>
		</div>
	}
}

#[component]
pub fn UserPage(tl: Timeline) -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Auth>().expect("missing auth context");
	let id = params.get().get("id").cloned().unwrap_or_default();
	let _id = id.clone(); // wtf triple clone??? TODO!!
	let actor = create_local_resource(move || _id.clone(), move |id| {
		async move {
			match CACHE.get(&Uri::full(FetchKind::User, &id)) {
				Some(x) => Some(x.clone()),
				None => {
					let user : serde_json::Value = Http::fetch(&Uri::api(FetchKind::User, &id, true), auth).await.ok()?;
					CACHE.put(Uri::full(FetchKind::User, &id), user.clone());
					Some(user)
				},
			}
		}
	});
	view! {
		<div>
			<Breadcrumb back=true >users::view</Breadcrumb>
			<div>
				{move || match actor.get() {
					None => view! { <p>loading...</p> }.into_view(),
					Some(None) => view! { <p><code>error loading</code></p> }.into_view(),
					Some(Some(object)) => {
						let uid = object.id().unwrap_or_default().to_string();
						let avatar_url = object.icon().get().map(|x| x.url().id().unwrap_or_default()).unwrap_or_default();
						let background_url = object.image().get().map(|x| x.url().id().unwrap_or_default()).unwrap_or_default();
						let display_name = object.name().unwrap_or_default().to_string();
						let username = object.preferred_username().unwrap_or_default().to_string();
						let summary = object.summary().unwrap_or_default().to_string();
						let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
						let actor_type = object.actor_type().unwrap_or(apb::ActorType::Person);
						let actor_type_tag = if actor_type == apb::ActorType::Person { None } else {
							Some(view! { <sup class="ml-s"><small>"["{actor_type.as_ref().to_lowercase()}"]"</small></sup> } )
						};
						let created = object.published();
						let following = object.following().get().map(|x| x.total_items().unwrap_or(0)).unwrap_or(0);
						let followers = object.followers().get().map(|x| x.total_items().unwrap_or(0)).unwrap_or(0);
						let statuses = object.outbox().get().map(|x| x.total_items().unwrap_or(0)).unwrap_or(0);
						let tl_url = format!("{}/outbox/page", Uri::api(FetchKind::User, &id.clone(), false));
						if !tl.next.get().starts_with(&tl_url) {
							tl.reset(tl_url);
						}
						view! {
							<div class="ml-3 mr-3">
								<div 
									class="banner"
									style={format!("background: center / cover url({background_url});")}
								>
									// <table class="align w-100">
									// <tr><td rowspan=3>
									// 	<img src=

									// </table>
									<div style="height: 10em"></div>
								</div>
								<div class="overlap">
									<table class="pl-2 pr-2 align w-100" style="table-layout: fixed">
										<tr>
											<td rowspan=4 style="width: 8em">
												<img class="avatar-circle avatar-border mr-s" src={avatar_url} style="height: 7em; width: 7em"/>
											</td>
											<td rowspan=2 class="bottom">
												<b class="big">{display_name}</b>{actor_type_tag}
											</td>
											<td rowspan=2 class="bottom rev" title="statuses">{statuses}" "<span class="emoji">"\u{1f582}"</span></td>
										</tr>
										<tr></tr>
										<tr>
											<td class="top">
												<small><a class="clean hover" href={uid} target="_blank">{username.clone()}@{domain}</a></small>
											</td>
											<td class="rev" title="following">{following}" "<span class="emoji">"👥"</span></td>
										</tr>
										<tr>
											<td>
												<DateTime t=created />
											</td>
											<td class="rev" title="followers">{followers}" "<span class="emoji">"📢"</span></td>
										</tr>
									</table>
									<blockquote class="ml-2 mt-1">{
									dissolve::strip_html_tags(&summary)
										.into_iter()
										.map(|x| view! { <div>{x}</div> })
										.collect_view()
									}</blockquote>
								</div>
							</div>
							<TimelineFeed tl=tl />
						}.into_view()
					},
				}}
			</div>
		</div>
	}
}

#[component]
pub fn ObjectPage(tl: Timeline) -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Auth>().expect("missing auth context");
	let object = create_local_resource(move || params.get().get("id").cloned().unwrap_or_default(), move |oid| {
		async move {
			match CACHE.get(&Uri::full(FetchKind::Object, &oid)) {
				Some(x) => Some(x.clone()),
				None => {
					let obj = Http::fetch::<serde_json::Value>(&Uri::api(FetchKind::Object, &oid, true), auth).await.ok()?;
					CACHE.put(Uri::full(FetchKind::Object, &oid), obj.clone());
					Some(obj)
				}
			}
		}
	});
	view! {
		<div>
			<Breadcrumb back=true >objects::view</Breadcrumb>
			<div class="ma-2" >
				{move || match object.get() {
					None => view! { <p> loading ... </p> }.into_view(),
					Some(None) => view! { <p><code>loading failed</code></p> }.into_view(),
					Some(Some(o)) => {
						let object = o.clone();
						let tl_url = format!("{}/page", Uri::api(FetchKind::Context, &o.context().id().unwrap_or_default(), false));
						if !tl.next.get().starts_with(&tl_url) {
							tl.reset(tl_url);
						}
						view!{
							<Object object=object />
							<div class="ml-1 mr-1 mt-2">
								<TimelineFeed tl=tl />
							</div>
						}.into_view()
					},
				}}
			</div>
		</div>
	}
}

#[component]
pub fn TimelinePage(name: &'static str, tl: Timeline) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<div>
			<Breadcrumb back=false>
				{name}
				<a class="clean ml-1" href="#" on:click=move |_| {
					tl.reset(tl.next.get().split('?').next().unwrap_or_default().to_string());
					spawn_local(async move {
						if let Err(e) = tl.more(auth).await {
							tracing::error!("error fetching more items for timeline: {e}");
						}
					})
				}><span class="emoji">
					"\u{1f5d8}"
				</span></a>
			</Breadcrumb>
			<div class="mt-s mb-s" >
				<TimelineFeed tl=tl />
			</div>
		</div>
	}
}

#[component]
pub fn DebugPage() -> impl IntoView {
	let url_ref: NodeRef<html::Input> = create_node_ref();
	let (object, set_object) = create_signal(serde_json::Value::String(
		"use this view to fetch remote AP objects and inspect their content".into())
	);
	let auth = use_context::<Auth>().expect("missing auth context");
	view! {
		<div>
			<Breadcrumb back=true>debug</Breadcrumb>
			<div class="mt-1" >
				<table class="align w-100" >
					<tr>
						<td><input class="w-100" type="text" node_ref=url_ref placeholder="AP id" /></td>
						<td>
							<input type="submit" class="w-100" value="fetch" on:click=move |_| {
								let fetch_url = url_ref.get().map(|x| x.value()).unwrap_or("".into());
								let url = format!("{URL_BASE}/dbg?id={fetch_url}");
								spawn_local(async move {
									match Http::fetch::<serde_json::Value>(&url, auth).await {
										Ok(x) => set_object.set(x),
										Err(e) => set_object.set(serde_json::Value::String(e.to_string())),
									}
								});
							} />
						</td>
					</tr>
				</table>
			</div>
			<pre class="ma-1" >
				{move || serde_json::to_string_pretty(&object.get()).unwrap_or("unserializable".to_string())}
			</pre>
		</div>
	}
}
