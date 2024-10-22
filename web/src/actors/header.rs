use leptos::*;
use leptos_router::*;
use crate::{getters::Getter, prelude::*, FALLBACK_IMAGE_URL};

use apb::{field::OptionalString, ActivityMut, Actor, Base, Object, ObjectMut};

#[component]
pub fn ActorHeader() -> impl IntoView {
	let params = use_params::<IdParam>();
	let auth = use_context::<Auth>().expect("missing auth context");
	let actor = create_local_resource(
		move || params.get().ok().and_then(|x| x.id).unwrap_or_default(),
		move |id| {
			async move {
				match cache::OBJECTS.get(&Uri::full(U::Actor, &id)) {
					Some(x) => Ok::<_, String>(x.clone()),
					None => {
						let user : serde_json::Value = Http::fetch(&Uri::api(U::Actor, &id, true), auth)
							.await
							.map_err(|e| e.to_string())?;
						let user = std::sync::Arc::new(user);
						let uid = Uri::full(U::Actor, &id);
						cache::OBJECTS.store(&uid, user.clone());
						if let Some(url) = user.url().id().str() {
							cache::WEBFINGER.store(&url, uid);
						}
						Ok(user)
					},
				}
			}
		}
	);
	move || match actor.get() {
		None => view! { <Loader /> }.into_view(),
		Some(Err(e)) => view! { <code class="center cw color">"could not resolve user: "{e}</code> }.into_view(),
		Some(Ok(actor)) => {
			let avatar_url = actor.icon().get().map(|x| x.url().id().str().unwrap_or(FALLBACK_IMAGE_URL.into())).unwrap_or(FALLBACK_IMAGE_URL.into());
			let background_url = actor.image().get().map(|x| x.url().id().str().unwrap_or(FALLBACK_IMAGE_URL.into())).unwrap_or(FALLBACK_IMAGE_URL.into());
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
			let fields = actor.attachment()
				.map(|x| view! {
					<tr>
						<td class="w-25"><b class="color">{x.name().str().unwrap_or_default()}</b></td>
						<td class="w-75" inner_html={mdhtml::safe_html(x.value().unwrap_or_default())}></td>
					</tr>
				})
				.collect_view();
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
									<img class="avatar avatar-border mr-s" src={avatar_url} style="height: 7em; width: 7em" onerror={format!("this.onerror=null; this.src='{FALLBACK_IMAGE_URL}';")} />
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
							{if following_me {
								Some(view! { <code class="mr-1 color">follows you</code> })
							} else {
								None
							}}
							{if followed_by_me {
								view! {
									<code class="color">"following"</code>
									<input type="submit" value="x" on:click=move |_| unfollow(_uid.clone()) />
								}.into_view()
							} else {
								view! { <input type="submit" value="follow" on:click=move |_| send_follow_request(_uid.clone()) /> }.into_view()
							}}
						</div>
					</div>
					<p class="mb-2 mt-0 center bio" inner_html={mdhtml::safe_html(actor.summary().unwrap_or_default())}></p>
					<p class="center">
						<table class="fields center w-100 pa-s" style="margin: auto; table-layout: fixed;">{fields}</table>
					</p>
				</div>
				<Outlet />
			}.into_view()
		},
	}
}

#[allow(unused)]
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

fn unfollow(target: String) {
	let auth = use_context::<Auth>().expect("missing auth context");
	spawn_local(async move {
		let payload = apb::new() 
			.set_activity_type(Some(apb::ActivityType::Undo))
			.set_to(apb::Node::links(vec![target.clone()]))
			.set_object(apb::Node::object(
				apb::new()
					.set_activity_type(Some(apb::ActivityType::Follow))
					.set_object(apb::Node::link(target))
			));
		if let Err(e) = Http::post(&auth.outbox(), &payload, auth).await {
			tracing::error!("failed sending follow request: {e}");
		}
	})
}
