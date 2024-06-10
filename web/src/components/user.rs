use leptos::*;
use crate::{prelude::*, DEFAULT_AVATAR_URL};

use apb::{field::OptionalString, Activity, ActivityMut, Actor, Base, Object, ObjectMut};

#[component]
pub fn ActorStrip(object: crate::Object) -> impl IntoView {
	let actor_id = object.id().unwrap_or_default().to_string();
	let username = object.preferred_username().unwrap_or_default().to_string();
	let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
	let avatar = object.icon().get().map(|x| x.url().id().str().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
	view! {
		<a href={Uri::web(U::Actor, &actor_id)} class="clean hover">
			<img src={avatar} class="avatar inline mr-s" /><b>{username}</b><small>@{domain}</small>
		</a>
	}
}

#[component]
pub fn ActorBanner(object: crate::Object) -> impl IntoView {
	match object.as_ref() {
		serde_json::Value::String(id) => view! {
			<div><b>?</b>" "<a class="clean hover" href={Uri::web(U::Actor, id)}>{Uri::pretty(id)}</a></div>
		},
		serde_json::Value::Object(_) => {
			let uid = object.id().unwrap_or_default().to_string();
			let uri = Uri::web(U::Actor, &uid);
			let avatar_url = object.icon().get().map(|x| x.url().id().str().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
			let display_name = object.name().unwrap_or_default().to_string();
			let username = object.preferred_username().unwrap_or_default().to_string();
			let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
			view! {
				<div>
					<table class="align" >
					<tr>
						<td rowspan="2" ><a href={uri.clone()} ><img class="avatar avatar-actor" src={avatar_url} /></a></td>
						<td><b>{display_name}</b></td>
					</tr>
					<tr>
						<td class="top" ><a class="hover" href={uri} ><small>{username}@{domain}</small></a></td>
					</tr>
					</table>
				</div>
			}
		},
		_ => view! {
			<div><b>invalid actor</b></div>
		}
	}
}

#[component]
pub fn FollowRequestButtons(activity_id: String, actor_id: String) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	// TODO lmao what is going on with this double move / triple clone ???????????
	let _activity_id = activity_id.clone();
	let _actor_id = actor_id.clone();
	let from_actor = CACHE.get(&activity_id).map(|x| x.actor().id().str().unwrap_or_default()).unwrap_or_default();
	let _from_actor = from_actor.clone();
	if actor_id == auth.user_id() {
		Some(view! {
			<input type="submit" value="accept"
				on:click=move |_| {
					let activity_id = _activity_id.clone();
					let actor_id = _from_actor.clone();
					spawn_local(async move {
						send_follow_response(
							apb::ActivityType::Accept(apb::AcceptType::Accept),
							activity_id,
							actor_id,
							auth
						).await
					})
				}
			/>
			<span class="ma-1"></span>
			<input type="submit" value="reject"
				on:click=move |_| {
					let activity_id = activity_id.clone();
					let actor_id = from_actor.clone();
					spawn_local(async move {
						send_follow_response(
							apb::ActivityType::Reject(apb::RejectType::Reject),
							activity_id,
							actor_id,
							auth
						).await
					})
				}
			/>
		})
	} else {
		None
	}
}

async fn send_follow_response(kind: apb::ActivityType, target: String, to: String, auth: Auth) {
	let payload = serde_json::Value::Object(serde_json::Map::default())
		.set_activity_type(Some(kind))
		.set_object(apb::Node::link(target))
		.set_to(apb::Node::links(vec![to]));
	if let Err(e) = Http::post(&auth.outbox(), &payload, auth).await {
		tracing::error!("failed posting follow response: {e}");
	}
}

#[component]
pub fn ActorHeader(object: crate::Object) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let avatar_url = object.icon().get().map(|x| x.url().id().str().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
	let background_url = object.image().get().map(|x| x.url().id().str().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
	let display_name = object.name().unwrap_or_default().to_string();
	let username = object.preferred_username().unwrap_or_default().to_string();
	let created = object.published().ok();
	let following = object.following_count().unwrap_or(0);
	let followers = object.followers_count().unwrap_or(0);
	let statuses = object.statuses_count().unwrap_or(0);
	let following_me = object.following_me().unwrap_or(false);
	let followed_by_me = object.followed_by_me().unwrap_or(false);
	let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
	let actor_type = object.actor_type().unwrap_or(apb::ActorType::Person);
	let actor_type_tag = if actor_type == apb::ActorType::Person { None } else {
		Some(view! { <sup class="ml-s"><small>"["{actor_type.as_ref().to_lowercase()}"]"</small></sup> } )
	};
	let uid = object.id().unwrap_or_default().to_string();
	let web_path = Uri::web(U::Actor, &uid);
	let _uid = uid.clone();
	view! {
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
							<img class="avatar avatar-border mr-s" src={avatar_url} style="height: 7em; width: 7em"/>
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
						<td class="rev" title="following">
							<a class="clean" href={format!("{web_path}/following")}>{following}" "<span class="emoji">"👥"</span></a>
						</td>
					</tr>
					<tr>
						<td>
							<DateTime t=created />
						</td>
						<td class="rev" title="followers">
							<a class="clean" href={format!("{web_path}/followers")}>{followers}" "<span class="emoji">"📢"</span></a>
						</td>
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
			</div>
	}.into_view()
}

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
