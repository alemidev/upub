use leptos::*;
use leptos_router::*;
use crate::prelude::*;
use crate::CONTACT;

use leptos_use::{signal_debounced, storage::use_local_storage, use_cookie_with_options, use_element_size, use_window_scroll, UseCookieOptions, utils::{FromToStringCodec, JsonCodec}, UseElementSizeReturn};

#[derive(Clone, Copy)]
pub struct Feeds {
	pub home: Timeline,
	pub global: Timeline,
	pub notifications: Timeline,
	// exploration feeds
	pub user: Timeline,
	pub server: Timeline,
	pub context: Timeline,
	pub tag: Timeline,
}

impl Feeds {
	pub fn new(username: &str) -> Self {
		Feeds {
			home: Timeline::new(format!("{URL_BASE}/actors/{username}/inbox/page")),
			notifications: Timeline::new(format!("{URL_BASE}/actors/{username}/notifications/page")),
			global: Timeline::new(format!("{URL_BASE}/inbox/page")),
			user: Timeline::new(format!("{URL_BASE}/actors/{username}/outbox/page")),
			server: Timeline::new(format!("{URL_BASE}/outbox/page")),
			tag: Timeline::new(format!("{URL_BASE}/tags/upub/page")),
			context: Timeline::new(format!("{URL_BASE}/outbox/page")), // TODO ehhh
		}
	}

	pub fn reset(&self) {
		self.home.reset(None);
		self.notifications.reset(None);
		self.global.reset(None);
		self.user.reset(None);
		self.server.reset(None);
		self.context.reset(None);
		self.tag.reset(None);
	}
}


#[component]
pub fn App() -> impl IntoView {
	let (token, set_token) = use_cookie_with_options::<String, FromToStringCodec>(
		"token",
		UseCookieOptions::default()
			.same_site(cookie::SameSite::Strict)
			// .secure(true)
			.path("/")
	);
	let (userid, set_userid) = use_cookie_with_options::<String, FromToStringCodec>(
		"user_id",
		UseCookieOptions::default()
			.same_site(cookie::SameSite::Strict)
			// .secure(true)
			.path("/")
	);
	let (config, set_config, _) = use_local_storage::<crate::Config, JsonCodec>("config");

	let auth = Auth { token, userid };

	let username = auth.userid.get_untracked()
		.map(|x| x.split('/').last().unwrap_or_default().to_string())
		.unwrap_or_default();

	let feeds = Feeds::new(&username);

	provide_context(auth);
	provide_context(config);
	provide_context(feeds);

	let reply_controls = ReplyControls::default();
	provide_context(reply_controls);

	let screen_width = document().body().map(|x| x.client_width()).unwrap_or_default();
	tracing::info!("detected width of {screen_width}");

	let (menu, set_menu) = create_signal(screen_width < 768);
	let (advanced, set_advanced) = create_signal(false);

	let title_target = move || if auth.present() { "/web/home" } else { "/web/global" };

	// refresh token immediately and  every hour
	let refresh = move || spawn_local(async move { Auth::refresh(auth.token, set_token, set_userid).await; });
	refresh();
	set_interval(refresh, std::time::Duration::from_secs(3600));

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
					<Navigator />
					<hr class="mt-1 mb-1" />
					<LoginBox
						token_tx=set_token
						userid_tx=set_userid
					/>
					<hr class="mt-1 mb-1" />
					<div class:hidden=move || !auth.present() >
						{move || if advanced.get() { view! {
							<AdvancedPostBox advanced=set_advanced/>
						}} else { view! {
							<PostBox advanced=set_advanced/>
						}}}
						<hr class="only-on-mobile sep mb-0 pb-0" />
					</div>
				</div>
				<div class="col-main" class:w-100=menu >
					<Router // TODO maybe set base="/web" ?
						trailing_slash=TrailingSlash::Redirect
						fallback=|| view! { <NotFound /> }
					>
						<main>
								<Routes>
									<Route path="/" view=move || view! { <Redirect path="/web" /> } />
									<Route path="/web" view=Scrollable >
										<Route path="" view=move ||
											if auth.present() {
												view! { <Redirect path="home" /> }
											} else {
												view! { <Redirect path="global" /> }
											}
										/>
										<Route path="home" view=move || view! { <Feed tl=feeds.home /> } />
										<Route path="global" view=move || view! { <Feed tl=feeds.global /> } />
										<Route path="local" view=move || view! { <Feed tl=feeds.server /> } />
										<Route path="notifications" view=move || view! { <Feed tl=feeds.notifications /> } />

										<Route path="about" view=AboutPage />
										<Route path="config" view=move || view! { <ConfigPage setter=set_config /> } />
										<Route path="dev" view=DebugPage />

										<Route path="actors/:id" view=ActorHeader > // TODO can we avoid this?
											<Route path="" view=ActorPosts />
											<Route path="following" view=move || view! { <FollowList outgoing=true /> } />
											<Route path="followers" view=move || view! { <FollowList outgoing=false /> } />
										</Route>

										<Route path="tags/:id" view=move || view! { <HashtagFeed tl=feeds.tag /> } />

										<Route path="objects/:id" view=ObjectView />
										// <Route path="/web/activities/:id" view=move || view! { <ActivityPage tl=context_tl /> } />

										<Route path="search" view=SearchPage />
										<Route path="register" view=RegisterPage />
									</Route>
								</Routes>
						</main>
					</Router>
				</div>
			</div>
		</div>
		<footer>
			<div class="sep-top">
				<span class="footer" >"\u{26fc} woven under moonlight  :: "<a class="clean" href="https://join.upub.social/" target="_blank" >about</a>" :: "<a class="clean" href=format!("mailto:{CONTACT}")>contact</a>" :: "<a class="clean" href="/web/dev">dev</a>" :: "<a class="clean" href="javascript:window.scrollTo({top:0, behavior:'smooth'})">top</a></span>
			</div>
		</footer>
	}
}

#[component]
fn Scrollable() -> impl IntoView {
	let location = use_location();
	let feeds = use_context::<Feeds>().expect("missing feeds context");
	let relevant_timeline = Signal::derive(move || {
		let path = location.pathname.get();
		if path.contains("/web/home") {
			Some(feeds.home)
		} else if path.contains("/web/global") {
			Some(feeds.global)
		} else if path.contains("/web/local") {
			Some(feeds.server)
		} else if path.starts_with("/web/notifications") {
			Some(feeds.notifications)
		} else if path.starts_with("/web/actors") {
			Some(feeds.user)
		} else if path.starts_with("/web/objects") {
			Some(feeds.context)
		} else {
			None
		}
	});
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
	let element = create_node_ref();
	let should_load = use_scroll_limit(element, 500.0);
	provide_context(should_load);
	view! {
		<div class="mb-1" node_ref=element>
			<div class="tl-header w-100 center mb-1">
				<a class="breadcrumb mr-1" href="javascript:history.back()" ><b>"<<"</b></a>
				<b>{crate::NAME}</b>" :: "{breadcrumb}
				{move || relevant_timeline.get().map(|tl| view! {
					<a class="breadcrumb ml-1" href="#" on:click=move|_| tl.refresh()  ><b>"↺"</b></a>
				})}
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
pub fn Loader() -> impl IntoView {
	view! {
		<div class="center mt-1 mb-1" >
			<button type="button" disabled>"loading "<span class="dots"></span></button>
		</div>
	}
}

pub fn use_scroll_limit<El, T>(el: El, offset: f64) -> Signal<bool>
where 
	El: Into<leptos_use::core::ElementMaybeSignal<T, web_sys::Element>> + Clone + 'static,
	T: Into<web_sys::Element> + Clone + 'static,
{
	let (load, set_load) = create_signal(false);
	let (_x, y) = use_window_scroll();
	let UseElementSizeReturn { height: screen_height, .. } = use_element_size("html");
	let UseElementSizeReturn { height, .. } = use_element_size(el);
	let scroll_state = Signal::derive(move || (y.get(), height.get(), screen_height.get()));
	let scroll_state_throttled = signal_debounced(
		scroll_state,
		50.
	);
	let _ = watch(
		move || scroll_state_throttled.get(),
		move |(y, height, screen), _, _| {
			let before = load.get();
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
