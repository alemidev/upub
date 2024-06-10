use leptos::*;
use leptos_router::*;
use crate::{getters::Getter, prelude::*, DEFAULT_AVATAR_URL};

use apb::{field::OptionalString, ActivityMut, Actor, Base, Object, ObjectMut};

#[component]
pub fn ActorHeader() -> impl IntoView {
	let params = use_params::<super::IdParam>();
	let auth = use_context::<Auth>().expect("missing auth context");
	let actor = create_local_resource(
		move || params.get().ok().and_then(|x| x.id).unwrap_or_default(),
		move |id| {
			async move {
				match CACHE.get(&Uri::full(U::Actor, &id)) {
					Some(x) => Ok::<_, String>(x.clone()),
					None => {
						let user : serde_json::Value = Http::fetch(&Uri::api(U::Actor, &id, true), auth)
							.await
							.map_err(|e| e.to_string())?;
						let user = std::sync::Arc::new(user);
						CACHE.put(Uri::full(U::Actor, &id), user.clone());
						Ok(user)
					},
				}
			}
		}
	);
	move || match actor.get() {
		None => view! { <Loader margin=true /> }.into_view(),
		Some(Err(e)) => view! { <code class="center cw color">"could not resolve user: "{e}</code> }.into_view(),
		Some(Ok(actor)) => {
			let avatar_url = actor.icon().get().map(|x| x.url().id().str().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
			let background_url = actor.image().get().map(|x| x.url().id().str().unwrap_or(DEFAULT_AVATAR_URL.into())).unwrap_or(DEFAULT_AVATAR_URL.into());
			let username = actor.preferred_username().unwrap_or_default().to_string();
			let name = actor.name().str().unwrap_or(username.clone());
			let created = actor.published().ok();
			let following_me = actor.following_me().unwrap_or(false);
			let followed_by_me = actor.followed_by_me().unwrap_or(false);
			let domain = actor.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
			let actor_type = actor.actor_type().unwrap_or(apb::ActorType::Person);
			let actor_type_tag = if actor_type == apb::ActorType::Person { None } else {
				Some(view! { <sup class="ml-s"><small>"["{actor_type.as_ref().to_lowercase()}"]"</small></sup> } )
			};
			let uid = actor.id().unwrap_or_default().to_string();
			let web_path = Uri::web(U::Actor, &uid);
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
									<img class="avatar avatar-border mr-s" src={avatar_url} style="height: 7em; width: 7em"/>
								</td>
								<td rowspan=2 class="bottom">
									<b class="big">{name}</b>{actor_type_tag}
								</td>
								<td rowspan=2 class="bottom rev" title="statuses">
									<a class="clean" href={web_path.clone()}>{actor.statuses_count().want()}" "<span class="emoji">"\u{1f582}"</span></a>
								</td>
							</tr>
							<tr></tr>
							<tr>
								<td class="top">
									<small><a class="clean hover" href={uid.clone()} target="_blank">{username.clone()}@{domain}</a></small>
								</td>
								<td class="rev" title="following">
									<a class="clean" href={format!("{web_path}/following")}>{actor.following_count().want()}" "<span class="emoji">"👥"</span></a>
								</td>
							</tr>
							<tr>
								<td>
									<DateTime t=created />
								</td>
								<td class="rev" title="followers">
									<a class="clean" href={format!("{web_path}/followers")}>{actor.followers_count().want()}" "<span class="emoji">"📢"</span></a>
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
					<p class="mb-2 mt-0 center" inner_html={mdhtml::safe_html(actor.summary().unwrap_or_default())}></p>
				</div>
				<Outlet />
			}.into_view()
		},
	}
}

async fn send_follow_response(kind: apb::ActivityType, target: String, to: String, auth: Auth) {
	let payload = apb::new()
		.set_activity_type(Some(kind))
		.set_object(apb::Node::link(target))
		.set_to(apb::Node::links(vec![to]));
	if let Err(e) = Http::post(&auth.outbox(), &payload, auth).await {
		tracing::error!("failed posting follow response: {e}");
	}
}

fn send_follow_request(target: String) {
	let auth = use_context::<Auth>().expect("missing auth context");
	spawn_local(async move {
		let payload = apb::new() 
			.set_activity_type(Some(apb::ActivityType::Follow))
			.set_object(apb::Node::link(target.clone()))
			.set_to(apb::Node::links(vec![target]));
		if let Err(e) = Http::post(&auth.outbox(), &payload, auth).await {
			tracing::error!("failed sending follow request: {e}");
		}
	})
}
