use apb::{Collection, Object};
use leptos::{either::Either, prelude::*};
use leptos_router::{components::*, hooks::{use_location, use_params}, path};
use crate::prelude::*;

use leptos_use::{
	signal_debounced, storage::use_local_storage, use_cookie_with_options, use_element_size, use_window_scroll,
	UseCookieOptions, UseElementSizeReturn
};

#[component]
pub fn App() -> impl IntoView {
	let (token, set_token) = use_cookie_with_options::<String, codee::string::FromToStringCodec>(
		"token",
		UseCookieOptions::default()
			.same_site(cookie::SameSite::Strict)
			// .secure(true)
			.path("/")
	);
	let (userid, set_userid) = use_cookie_with_options::<String, codee::string::FromToStringCodec>(
		"user_id",
		UseCookieOptions::default()
			.same_site(cookie::SameSite::Strict)
			// .secure(true)
			.path("/")
	);
	let (config, set_config, _) = use_local_storage::<crate::Config, codee::string::JsonSerdeCodec>("config");

	let (privacy, set_privacy) = signal(Privacy::Private);

	let auth = Auth { token, userid };

	let (be_version, set_be_version) = signal("?.?.?".to_string());
	leptos::task::spawn_local(async move {
		match Http::fetch::<serde_json::Value>(&format!("{URL_BASE}/nodeinfo/2.0.json"), auth).await {
			Err(e) => tracing::error!("failed fetching backend version: {e} - {e:?}"),
			Ok(nodeinfo) => {
				if let Some(version) = nodeinfo
					.get("software")
					.and_then(|x| x.get("version"))
					.and_then(|x| x.as_str())
				{
					set_be_version.set(version.to_string());
				}
			},
		}
	});

	provide_context(auth);
	provide_context(config);
	provide_context(privacy);

	let reply_controls = ReplyControls::default();
	provide_context(reply_controls);

	let screen_width = document().body().map(|x| x.client_width()).unwrap_or_default();
	tracing::info!("detected width of {screen_width}");

	let (menu, set_menu) = signal(screen_width < 768);
	let (advanced, set_advanced) = signal(false);

	let title_target = move || if auth.present() { "/web/home" } else { "/web/global" };

	// refresh token immediately and  every hour
	let refresh_token = move || leptos::task::spawn_local(async move { Auth::refresh(auth, set_token, set_userid).await; });
	refresh_token();
	set_interval(refresh_token, std::time::Duration::from_secs(3600));

	// refresh notifications
	let (notifications, set_notifications) = signal(0);
	let fetch_notifications = move || leptos::task::spawn_local(async move {
		if let Some(actor_id) = userid.get_untracked() {
			let notif_url = format!("{actor_id}/notifications");
			match Http::fetch::<serde_json::Value>(&notif_url, auth).await {
				Err(e) => tracing::error!("failed fetching notifications: {e}"),
				Ok(doc) => if let Ok(count) = doc.total_items() {
					set_notifications.set(count);
				},
			} 
		}
	});
	fetch_notifications();
	set_interval(fetch_notifications, std::time::Duration::from_secs(60));
	provide_context((notifications, set_notifications));

	view! {
		<nav class="w-100 mt-1 mb-1 pb-s">
			<code class="color ml-3" ><a class="upub-title" href=title_target >μpub</a></code>
			<small class="ml-1 mr-1 hidden-on-tiny" ><a class="clean" href="/web/global" >micro social network, federated</a></small>
			/* TODO kinda jank with the float but whatever, will do for now */
			<input type="submit" class="mr-2 rev" on:click=move |_| set_menu.set(!menu.get()) value="menu" style="float: right" />
		</nav>
		<hr class="sep sticky" />
		<div class="container mt-2 pt-2" >
			<div class="two-col" >
				<div class="col-side sticky pb-s" class:hidden=menu >
					<Navigator notifications=notifications />
					<hr class="mt-1 mb-1" />
					<LoginBox
						token_tx=set_token
						userid_tx=set_userid
					/>
					<hr class="mt-1 mb-1" />
					<div class:hidden=move || !auth.present() >
						<PrivacySelector setter=set_privacy />
						<hr class="mt-1 mb-1" />
						{move || if advanced.get() { Either::Left(view! {
							<AdvancedPostBox advanced=set_advanced/>
						})} else { Either::Right(view! {
							<PostBox advanced=set_advanced/>
						})}}
						<hr class="only-on-mobile sep mb-0 pb-0" />
					</div>
				</div>
				<div class="col-main" class:w-100=menu >
					<Router>
						<main>
								<Routes fallback=NotFound>
									<Route path=path!("/") view=move || view! { <Redirect path="/web" /> } />
									<ParentRoute path=path!("/web") view=Scrollable >
										<Route path=path!("") view=move ||
											if auth.present() {
												view! { <Redirect path="home" /> }
											} else {
												view! { <Redirect path="local" /> }
											}
										/>

										// main timelines
										<Route path=path!("home") view=move || if auth.present() {
											Either::Left(view! {
												<Loadable
													base=format!("{URL_BASE}/actors/{}/inbox/page", auth.username())
													element=move |obj| view! { <Item item=obj sep=true /> }
												/>
											}) 
										} else {
											Either::Right(view! { <Unauthorized /> })
										} />

										<Route path=path!("global") view=move || view! {
											<Loadable
												base=format!("{URL_BASE}/inbox/page")
												element=move |obj| view! { <Item item=obj sep=true /> }
											/>
										} />

										<Route path=path!("local") view=move || view! {
											<Loadable
												base=format!("{URL_BASE}/outbox/page")
												element=move |obj| view! { <Item item=obj sep=true /> }
											/>
										} />

										<Route path=path!("notifications") view=move || if auth.present() {
											Either::Left(view! {
												<Loadable
													base=format!("{URL_BASE}/actors/{}/notifications/page", auth.username())
													element=move |obj| view! { <Item item=obj sep=true always=true /> }
												/>
											})
										} else {
											Either::Right(view! { <Unauthorized /> })
										} />

										<Route path=path!("tags/:id") view=move || {
											let params = use_params::<IdParam>();
											let tag = params.get().ok().and_then(|x| x.id).unwrap_or_default();
											view! {
												<Loadable
													base=format!("{URL_BASE}/tag/{tag}/page", )
													element=move |obj| view! { <Item item=obj sep=true always=true /> }
												/>
											}
										} />

										<Route path=path!("groups") view=move || view! {
											<Loadable
												base=format!("{URL_BASE}/groups/page")
												convert=U::Actor
												element=|obj| view! { <ActorBanner object=obj /><hr/> }
											/>
										} />

										// static pages, configs and tools
										<Route path=path!("about") view=AboutPage />
										<Route path=path!("config") view=move || view! { <ConfigPage setter=set_config /> } />
										<Route path=path!("explore") view=DebugPage />
										<Route path=path!("search") view=SearchPage />
										<Route path=path!("register") view=RegisterPage />

										// actors
										<ParentRoute path=path!("actors/:id") view=ActorHeader > // TODO can we avoid this?
											<Route path=path!("") view=ActorPosts />
											<Route path=path!("likes") view=ActorLikes />
											<Route path=path!("following") view=move || view! { <FollowList outgoing=true /> } />
											<Route path=path!("followers") view=move || view! { <FollowList outgoing=false /> } />
										</ParentRoute>

										// objects
										<ParentRoute path=path!("objects/:id") view=ObjectView >
											<Route path=path!("") view=move || {
												let params = use_params::<IdParam>();
												let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
												let oid = Uri::full(U::Object, &id);
												let context_id = crate::cache::OBJECTS.get(&oid)
													.and_then(|obj| obj.context().id().ok())
													.unwrap_or(oid.clone());
												view! {
													<Loadable
														base=format!("{}/context/page", Uri::api(U::Object, &context_id, false))
														convert=U::Object
														element=move |obj| view! { <Item item=obj always=true slim=true /> }
														thread=oid
													/>
												}
											} />
											<Route path=path!("replies") view=move || {
												let params = use_params::<IdParam>();
												let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
												let oid = Uri::full(U::Object, &id);
												view! {
													<Loadable
														base=format!("{}/replies/page", Uri::api(U::Object, &oid, false))
														convert=U::Object
														element=move |obj| view! { <Item item=obj always=true slim=true /> }
													/>
												}
											} />
											<Route path=path!("likes") view=move || {
												let params = use_params::<IdParam>();
												let id = params.get().ok().and_then(|x| x.id).unwrap_or_default();
												let oid = Uri::full(U::Object, &id);
												view! {
													<Loadable
														base=format!("{}/likes/page", Uri::api(U::Object, &oid, false))
														element=move |obj| view! { <Item item=obj always=true /> }
													/>
												}
											} />
											// <Route path="announced" view=ObjectAnnounced />
										</ParentRoute>

										// TODO a standalone way to view activities?
										// <Route path="/web/activities/:id" view=move || view! { <ActivityPage tl=context_tl /> } />
									</ParentRoute>
								</Routes>
						</main>
					</Router>
				</div>
			</div>
		</div>
		<footer>
			<div class="sep-top">
				<span class="footer" >"\u{26fc} woven under moonlight :: "<a class="clean" href="https://join.upub.social/" target="_blank" >"μpub"</a>" :: FE v"{crate::VERSION}" :: BE v"{be_version}" :: "<a class="clean" href="javascript:window.scrollTo({top:0, behavior:'smooth'})">top</a></span>
			</div>
		</footer>
	}
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FeedRoute {
	Unknown, Home, Global, Server, Notifications, User, Following, Followers, ActorLikes, ObjectLikes, Replies, Context
}

impl FeedRoute {
	fn is_refreshable(&self) -> bool {
		!matches!(self, Self::Unknown)
	}
}

#[component]
fn Scrollable() -> impl IntoView {
	let location = use_location();
	// TODO this is terrible!! omg maybe it should receive from context current timeline?? idk this
	//      is awful and i patched it another time instead of doing it properly...
	//      at least im going to provide a route enum to use in other places
	// UPDATE it's a bit less terrible since now we just update an enum but still probs should do
	//        something fancier than this string stuff... now leptos has path segments as structs,
	//        maybe maybe maybe it's accessible to us??
	let (route, set_route) = signal(FeedRoute::Unknown);
	let _ = Effect::watch(
		move || location.pathname.get(),
		move |path, _path_prev, _| {
			if path.contains("/web/home") {
				set_route.set(FeedRoute::Home);
			} else if path.contains("/web/global") {
				set_route.set(FeedRoute::Global);
			} else if path.contains("/web/local") {
				set_route.set(FeedRoute::Server);
			} else if path.starts_with("/web/notifications") {
				set_route.set(FeedRoute::Notifications);
			} else if path.starts_with("/web/actors") {
				match path.split('/').nth(4) {
					Some("following") => {
						set_route.set(FeedRoute::Following);
					},
					Some("followers") => {
						set_route.set(FeedRoute::Followers);
					},
					Some("likes") => {
						set_route.set(FeedRoute::ActorLikes);
					},
					_ => {
						set_route.set(FeedRoute::User);
					},
				}
			} else if path.starts_with("/web/objects") {
				match path.split('/').nth(4) {
					Some("likes") => {
						set_route.set(FeedRoute::ObjectLikes);
					},
					Some("replies") => {
						set_route.set(FeedRoute::Replies);
					},
					_ => {
						set_route.set(FeedRoute::Context);
					},
				}
			} else {
				set_route.set(FeedRoute::Unknown);
			}
		},
		true
	);
	provide_context(route);
	let breadcrumb = Signal::derive(move || {
		let path = location.pathname.get();
		let mut path_iter = path.split('/').skip(2);
		// TODO wow this breadcrumb logic really isnt nice can we make it better??
		match path_iter.next() {
			Some("actors") => match path_iter.next() {
				None => "actors :: all".to_string(),
				Some(id) => {
					let mut out = "actors :: ".to_string();
					if id.starts_with('+') {
						out.push_str("proxy");
					} else {
						out.push_str(id);
					}
					if let Some(x) = path_iter.next() {
						out.push_str(" :: ");
						out.push_str(x);
					}
					out
				},
			},
			Some("tags") => format!("tags :: {}", path_iter.next().unwrap_or_default()),
			Some(p) => p.to_string(),
			None => "?".to_string(),
		}
	});
	let element = NodeRef::new();
	let should_load = use_scroll_limit(element, 500.0);
	provide_context(should_load);
	let (refresh, set_refresh) = signal(());
	provide_context(refresh);
	provide_context(set_refresh);
	view! {
		<div class="mb-1" node_ref=element>
			<div class="tl-header w-100 center mb-1">
				<a class="breadcrumb mr-1" href="javascript:history.back()" ><b>"<<"</b></a>
				<b>{crate::NAME}</b>" :: "{breadcrumb}
				{move || if route.get().is_refreshable() {
					Some(view! {
						<a class="breadcrumb ml-1" href="#" on:click=move|_| set_refresh.set(())  ><b>"↺"</b></a>
					})
				} else {
					None
				}}
			</div>
			<Outlet />
		</div>
	}
}

#[component]
pub fn NotFound() -> impl IntoView {
	view! {
		<div class="center">
			<h3>nothing to see here!</h3>
			<p><a href="/web"><button type="button">back to root</button></a></p>
		</div>
	}
}

#[component]
pub fn Unauthorized() -> impl IntoView {
	view! {
		<p>
			<code class="color center cw">please log in first</code>
		</p>
	}
}

#[component]
//#[deprecated = "should not be displaying this directly"]
pub fn Loader() -> impl IntoView {
	view! {
		<div class="center mt-1 mb-1" >
			<button type="button" disabled>"loading "<span class="dots"></span></button>
		</div>
	}
}

pub fn use_scroll_limit<T, Marker>(el: NodeRef<T>, offset: f64) -> Signal<bool>
where
	T: leptos::html::ElementType,
	NodeRef<T>: leptos_use::core::IntoElementMaybeSignal<web_sys::Element, Marker>,
{
	let (load, set_load) = signal(false);
	let (_x, y) = use_window_scroll();
	let UseElementSizeReturn { height: screen_height, .. } = use_element_size(document().document_element().expect("could not get DOM"));
	let UseElementSizeReturn { height, .. } = use_element_size(el);
	let scroll_state = Signal::derive(move || (y.get(), height.get(), screen_height.get()));
	let scroll_state_throttled = signal_debounced(
		scroll_state,
		50.
	);
	let _ = Effect::watch(
		move || scroll_state_throttled.get(),
		move |(y, height, screen), _, _| {
			let before = load.get_untracked();
			let after = *height <= *screen || y + screen + offset >= *height;
			let force = *y + screen >= *height;
			if force || after != before || *height < *screen {
				set_load.set(after)
			}
		},
		false,
	);
	load.into()
}
