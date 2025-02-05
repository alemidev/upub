use leptos::prelude::*;
use crate::{prelude::*, FALLBACK_IMAGE_URL};

use apb::{Activity, ActivityMut, Actor, Base, Object, ObjectMut, Shortcuts};

lazy_static::lazy_static! {
	static ref REGEX: regex::Regex = regex::Regex::new(r":\w+:").expect("failed compiling custom emoji regex");
}

#[component]
pub fn ActorStrip(object: crate::Doc) -> impl IntoView {
	let actor_id = object.id().unwrap_or_default().to_string();
	let username = object.preferred_username().unwrap_or_default().to_string();
	let domain = object.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
	let avatar = object.icon_url().unwrap_or(FALLBACK_IMAGE_URL.into());
	view! {
		<a href={Uri::web(U::Actor, &actor_id)} class="clean hover">
			<img src={avatar} class="avatar inline mr-s" onerror={format!("this.onerror=null; this.src='{FALLBACK_IMAGE_URL}';")} /><b>{username}</b><small>@{domain}</small>
		</a>
	}
}

#[component]
pub fn ActorBanner(object: crate::Doc) -> impl IntoView {
	match object.as_ref() {
		serde_json::Value::String(id) => view! {
			<div><b>?</b>" "<a class="clean hover" href={Uri::web(U::Actor, id)}>{Uri::pretty(id, 50)}</a></div>
		}.into_any(),
		serde_json::Value::Object(_) => {
			let uid = object.id().unwrap_or_default().to_string();
			let uri = Uri::web(U::Actor, &uid);
			let avatar_url = object.icon_url().unwrap_or(FALLBACK_IMAGE_URL.into());
			let username = object.preferred_username().unwrap_or_default().to_string();
			let domain = object.id().unwrap_or_default().replace("https://", "").replace("http://", "").split('/').next().unwrap_or_default().to_string();
			let display_name = object.name().unwrap_or_default().to_string();
			view! {
				<div>
					<table class="align" >
					<tr>
						<td rowspan="2" ><a href={uri.clone()} ><img class="avatar avatar-actor" src={avatar_url} onerror={format!("this.onerror=null; this.src='{FALLBACK_IMAGE_URL}';")} /></a></td>
						<td><b class="displayname"><DisplayName name=display_name /></b></td>
					</tr>
					<tr>
						<td class="top" ><a class="hover" href={uri} ><small>{username}@{domain}</small></a></td>
					</tr>
					</table>
				</div>
			}.into_any()
		},
		_ => view! {
			<div><b>invalid actor</b></div>
		}.into_any()
	}
}

#[component]
fn DisplayName(mut name: String) -> impl IntoView {
	for m in REGEX.find_iter(&name.clone()) {
		// TODO this is a clear unmitigated unsanitized html injection ahahahahaha but accounts 
		//      with many custom emojis in their names mess with my frontend and i dont want to 
		//      deal with it rn
		name = name.replace(m.as_str(), &format!("<u class=\"moreinfo\" title=\"{}\">[::]</u>", m.as_str()));
	}
	view! { <span inner_html=name></span> }
}

#[component]
pub fn FollowRequestButtons(activity_id: String, actor_id: String) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	// TODO lmao what is going on with this double move / triple clone ???????????
	let _activity_id = activity_id.clone();
	let _actor_id = actor_id.clone();
	let from_actor = cache::OBJECTS.get(&activity_id).and_then(|x| x.actor().id().ok()).unwrap_or_default();
	let _from_actor = from_actor.clone();
	if actor_id == auth.user_id() {
		Some(view! {
			<input type="submit" value="accept"
				on:click=move |_| {
					let activity_id = _activity_id.clone();
					let actor_id = _from_actor.clone();
					leptos::task::spawn_local(async move {
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
					leptos::task::spawn_local(async move {
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
