use std::sync::Arc;

use leptos::*;
use leptos_router::*;
use crate::{prelude::*, DEFAULT_AVATAR_URL};

use apb::{Base, Actor, ActivityMut, Object, ObjectMut};

fn send_follow_request(target: String) {
	let auth = use_context::<Auth>().expect("missing auth context");
	spawn_local(async move {
		let payload = serde_json::Value::Object(serde_json::Map::default())
			.set_activity_type(Some(apb::ActivityType::Follow))
			.set_object(apb::Node::link(target.clone()))
			.set_to(apb::Node::links(vec![target]));
		if let Err(e) = Http::post(&auth.outbox(), &payload, auth).await {
			tracing::error!("failed sending follow request: {e}");
		}
	})
}

#[component]
pub fn UserPage(tl: Timeline) -> impl IntoView {
	let params = use_params_map();
	let auth = use_context::<Auth>().expect("missing auth context");
	let id = params.get()
		.get("id")
		.cloned()
		.unwrap_or_default();
	let mut uid = id
		.replace("/web/objects/", "")
		.replacen('+', "https://", 1)
		.replace('@', "/");
	if !uid.starts_with("http") {
		uid = format!("{URL_BASE}/web/objects/{uid}");
	}
	let actor = create_local_resource(move || params.get().get("id").cloned().unwrap_or_default(), move |id| {
		async move {
			match CACHE.get(&Uri::full(FetchKind::User, &id)) {
				Some(x) => Some(x.clone()),
				None => {
					let user : serde_json::Value = Http::fetch(&Uri::api(FetchKind::User, &id, true), auth).await.ok()?;
					let user = Arc::new(user);
					CACHE.put(Uri::full(FetchKind::User, &id), user.clone());
					Some(user)
				},
			}
		}
	});
	view! {
		<div>
			<Breadcrumb back=true >
				users::view
				<a
					class="clean ml-1" href="#"
					class:hidden=move || tl.is_empty()
					on:click=move |_| {
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
			<div>
				{move || {
					let uid = uid.clone();
					match actor.get() {
						None => view! { <p class="center">loading...</p> }.into_view(),
						Some(None) => {
							view! { <p class="center"><code>loading failed</code><sup><small><a class="clean" href={uid} target="_blank">"↗"</a></small></sup></p> }.into_view()
						},
						Some(Some(object)) => {
							let uid = object.id().unwrap_or_default().to_string();
							let avatar_url = object.icon().get().map(|x| x.url().id().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
							let background_url = object.image().get().map(|x| x.url().id().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
							let display_name = object.name().unwrap_or_default().to_string();
							let username = object.preferred_username().unwrap_or_default().to_string();
							let summary = object.summary().unwrap_or_default().to_string();
							let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
							let actor_type = object.actor_type().unwrap_or(apb::ActorType::Person);
							let actor_type_tag = if actor_type == apb::ActorType::Person { None } else {
								Some(view! { <sup class="ml-s"><small>"["{actor_type.as_ref().to_lowercase()}"]"</small></sup> } )
							};
							let created = object.published();
							let following = object.following_count().unwrap_or(0);
							let followers = object.followers_count().unwrap_or(0);
							let statuses = object.statuses_count().unwrap_or(0);
							let tl_url = format!("{}/outbox/page", Uri::api(FetchKind::User, &id.clone(), false));
							if !tl.next.get().starts_with(&tl_url) {
								tl.reset(tl_url);
							}
							let following_me = object.following_me().unwrap_or(false);
							let followed_by_me = object.followed_by_me().unwrap_or(false);
							let _uid = uid.clone();

							view! {
								<div class="ml-3 mr-3">
									<div 
										class="banner"
										style={format!("background: center / cover url({background_url});")}
									>
										<div style="height: 10em"></div> // TODO bad way to have it fixed height ewwww
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
													<small><a class="clean hover" href={uid.clone()} target="_blank">{username.clone()}@{domain}</a></small>
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
										<div class="rev mr-1" class:hidden=move || !auth.present() || auth.user_id() == uid>
											{if followed_by_me {
												view! { <code class="color">following</code> }.into_view()
											} else {
												view! { <input type="submit" value="follow" on:click=move |_| send_follow_request(_uid.clone()) /> }.into_view()
											}}
											{if following_me {
												Some(view! { <code class="ml-1 color">follows you</code> })
											} else {
												None
											}}
										</div>
										<p class="ml-2 mt-1 center" inner_html={mdhtml::safe_html(&summary)}></p>
									</div>
								</div>
								<TimelineFeed tl=tl />
							}.into_view()
						},
					}
				}}
			</div>
		</div>
	}
}