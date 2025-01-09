use leptos::*;
use leptos_router::*;
use crate::{app::FeedRoute, prelude::*, FALLBACK_IMAGE_URL};

use apb::{ActivityMut, Actor, Base, Object, ObjectMut, Shortcuts};

#[component]
pub fn ActorHeader() -> impl IntoView {
	let params = use_params::<IdParam>();
	let auth = use_context::<Auth>().expect("missing auth context");
	let config = use_context::<Signal<crate::Config>>().expect("missing config context");
	let relevant_tl = use_context::<Signal<Option<Timeline>>>().expect("missing relevant timeline context");
	let matched_route = use_context::<ReadSignal<crate::app::FeedRoute>>().expect("missing route context");
	let (loading, set_loading) = create_signal(false);
	let actor = create_local_resource(
		move || params.get().ok().and_then(|x| x.id).unwrap_or_default(),
		move |id| {
			async move {
				match cache::OBJECTS.get(&Uri::full(U::Actor, &id)) {
					Some(x) => Some(x.clone()),
					None => {
						let user = cache::OBJECTS.resolve(&id, U::Actor, auth).await?;
						Some(user)
					},
				}
			}
		}
	);
	move || match actor.get() {
		None => view! { <Loader /> }.into_view(),
		Some(None) => view! { <code class="center cw color">"could not resolve user"</code> }.into_view(),
		Some(Some(actor)) => {
			let avatar_url = actor.icon_url().unwrap_or(FALLBACK_IMAGE_URL.into());
			let background_url = actor.image_url().unwrap_or(FALLBACK_IMAGE_URL.into());
			let username = actor.preferred_username().unwrap_or_default().to_string();
			let name = actor.name().unwrap_or(username.clone());
			let created = actor.published().ok();
			let following_me = actor.following_me().unwrap_or(false);
			let followed_by_me = actor.followed_by_me().unwrap_or(false);
			let domain = actor.id().unwrap_or_default().replace("https://", "").split('/').next().unwrap_or_default().to_string();
			let actor_type = actor.actor_type().unwrap_or(apb::ActorType::Person);
			let actor_type_tag = if actor_type == apb::ActorType::Person { None } else {
				Some(view! { <sup class="ml-s"><small>"["{actor_type.as_ref().to_lowercase()}"]"</small></sup> } )
			};
			let fields = actor.attachment()
				.flat()
				.into_iter()
				.filter_map(|x| x.into_inner().ok())
				.map(|x| view! {
					<tr>
						<td class="w-25"><b class="color">{x.name().unwrap_or_default()}</b></td>
						<td class="w-75" inner_html={mdhtml::safe_html(&x.value().unwrap_or_default())}></td>
					</tr>
				})
				.collect_view();
			let uid = actor.id().unwrap_or_default().to_string();
			let web_path = Uri::web(U::Actor, &uid);
			// TODO what the fuck...
			let _uid = uid.clone();
			let __uid = uid.clone();
			view! {
				<div class="ml-3 mr-3">
					<div 
						class="banner"
						style={format!("background: center / cover url({background_url});")}
					>
						<div style="height: 10em"></div> // TODO bad way to have it fixed height ewwww
					</div>
					<div class="overlap">
						<div class="pl-1 pr-1" style="display: flex">
							<img class="avatar avatar-border mr-s" src={avatar_url} style="height: 7em; width: 7em" onerror={format!("this.onerror=null; this.src='{FALLBACK_IMAGE_URL}';")} />

							<div class="ma-s">
								<p class="line shadow">
									<span class="emoji mr-s">"\u{1f582}"</span><small>{actor.statuses_count().unwrap_or_default()}</small>
									<span class="emoji ml-1 mr-s">"👥"</span><small>{actor.following_count().unwrap_or_default()}</small>
									<span class="emoji ml-1 mr-s">"📢"</span><small>{actor.followers_count().unwrap_or_default()}</small>
								</p>
								<p class="line pt-1"><b class="big mt-1">{name}</b>{actor_type_tag}</p>
								<p class="line"><small><a class="clean hover" href={uid.clone()} target="_blank">{username.clone()}@{domain}</a></small></p>
								<p class="line"><DateTime t=created /></p>
							</div>
							<div class="ma-s pt-3 rev" style="flex-grow: 1; white-space: nowrap;">
							</div>
						</div>
						<div class="mr-1 ml-1" class:hidden=move || !auth.present() || auth.user_id() == uid>
							{if following_me {
								Some(view! {
									<a class="clean dim" href="#remove" on:click=move |_| tracing::error!("not yet implemented")>
										<span class="border-button ml-s" title="remove follower (not yet implemented)">
											<code class="color mr-s">"!"</code>
											<small class="mr-s">follows you</small>
										</span>
									</a>
								})
							} else {
								None
							}}
							{if followed_by_me {
								view! {
									<a class="clean dim" href="#unfollow" on:click=move |_| unfollow(_uid.clone())>
										<span class="border-button ml-s" title="undo follow">
											<code class="color mr-s">x</code>
											<small class="mr-s">following</small>
										</span>
									</a>
								}.into_view()
							} else {
								view! {
									<a class="clean dim" href="#follow" on:click=move |_| send_follow_request(_uid.clone())>
										<span class="border-button ml-s" title="send follow request">
											<code class="color mr-s">+</code>
											<small class="mr-s">follow</small>
										</span>
									</a>
								}.into_view()
							}}
						</div>
					</div>
					<p class="mb-2 mt-0 center bio" inner_html={mdhtml::safe_html(&actor.summary().unwrap_or_default())}></p>
					<p class="center">
						<table class="fields center w-100 pa-s" style="margin: auto; table-layout: fixed;">{fields}</table>
					</p>
				</div>
				<p class="mt-2">
					<span class:tab-active=move || matches!(matched_route.get(), FeedRoute::User)>
						<a class="clean" href=web_path.clone()><span class="emoji">"🖂 "</span>"outbox"</a>
					</span>
					<span class="ml-1" class:tab-active=move || matches!(matched_route.get(), FeedRoute::Likes)>
						<a class="clean" href=format!("{web_path}/likes")><span class="emoji">"⭐ "</span>"likes"</a>
					</span>
					<span class="ml-1" style="float: right" class:tab-active=move || matches!(matched_route.get(), FeedRoute::Followers)>
						<a class="clean" href=format!("{web_path}/followers")><span class="emoji">"📢"</span><span class:hidden-on-mobile=move || !matches!(matched_route.get(), FeedRoute::Followers)>" followers"</span></a>
					</span>
					<span class="ml-1" style="float: right" class:tab-active=move || matches!(matched_route.get(), FeedRoute::Following)>
						<a class="clean" href=format!("{web_path}/following")><span class="emoji">"👥"</span><span class:hidden-on-mobile=move || !matches!(matched_route.get(), FeedRoute::Following)>" following"</span></a>
					</span>
					{move || if auth.present() {
						if loading.get() {
							Some(view! {
								<span style="float: right">
									<span class="hidden-on-mobile">"fetching "</span><span class="dots"></span>
								</span>
							})
						} else {
							let uid = __uid.clone();
							Some(view! {
								<span style="float: right">
									<a
										class="clean"
										on:click=move |ev| fetch_cb(ev, set_loading, uid.clone(), auth, config, relevant_tl)
										href="#"
									>
										<span class="emoji ml-2">"↺ "</span><span class="hidden-on-mobile">"fetch"</span>
									</a>
								</span>
							})
						}
					} else {
						None
					}}
				</p>
				<hr class="color" />
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

fn fetch_cb(ev: ev::MouseEvent, set_loading: WriteSignal<bool>, uid: String, auth: Auth, config: Signal<crate::Config>, relevant_tl: Signal<Option<Timeline>>) {
	let api = Uri::api(U::Actor, &uid, false);
	ev.prevent_default();
	set_loading.set(true);
	spawn_local(async move {
		if let Err(e) = Http::fetch::<serde_json::Value>(&format!("{api}/outbox?fetch=true"), auth).await {
			tracing::error!("failed fetching outbox for {uid}: {e}");
		}
		set_loading.set(false);
		relevant_tl.get().inspect(|x| x.refresh(auth, config));
	});
}
