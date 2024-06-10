use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use leptos_use::{storage::use_local_storage, use_cookie, utils::{FromToStringCodec, JsonCodec}};

#[derive(Clone, Copy)]
pub struct Feeds {
	// object feeds
	pub home: Timeline,
	pub global: Timeline,
	// notification feeds
	pub private: Timeline,
	pub public: Timeline,
	// exploration feeds
	pub user: Timeline,
	pub server: Timeline,
	pub context: Timeline,
}

impl Feeds {
	pub fn new(username: &str) -> Self {
		Feeds {
			home: Timeline::new(format!("{URL_BASE}/actors/{username}/feed/page")),
			global: Timeline::new(format!("{URL_BASE}/feed/page")),
			private: Timeline::new(format!("{URL_BASE}/actors/{username}/inbox/page")),
			public: Timeline::new(format!("{URL_BASE}/inbox/page")),
			user: Timeline::new(format!("{URL_BASE}/actors/{username}/outbox/page")),
			server: Timeline::new(format!("{URL_BASE}/outbox/page")),
			context: Timeline::new(format!("{URL_BASE}/outbox/page")), // TODO ehhh
		}
	}

	pub fn reset(&self) {
		self.home.reset(None);
		self.global.reset(None);
		self.private.reset(None);
		self.public.reset(None);
		self.user.reset(None);
		self.server.reset(None);
		self.context.reset(None);
	}
}


#[component]
pub fn App() -> impl IntoView {
	let (token, set_token) = use_cookie::<String, FromToStringCodec>("token");
	let (userid, set_userid) = use_cookie::<String, FromToStringCodec>("user_id");
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

	let screen_width = window().screen().map(|x| x.avail_width().unwrap_or_default()).unwrap_or_default();

	let (menu, set_menu) = create_signal(screen_width <= 786);
	let (advanced, set_advanced) = create_signal(false);

	let title_target = move || if auth.present() { "/web/home" } else { "/web/server" };

	spawn_local(async move {
		// refresh token first, or verify that we're still authed
		if Auth::refresh(auth.token, set_token, set_userid).await {
			feeds.home.more(auth); // home inbox requires auth to be read
			feeds.private.more(auth);
		}
		feeds.global.more(auth);
		feeds.public.more(auth); // server inbox may contain private posts
	});


	// refresh token every hour
	set_interval(
		move || spawn_local(async move { Auth::refresh(auth.token, set_token, set_userid).await; }),
		std::time::Duration::from_secs(3600)
	);

	view! {
		<nav class="w-100 mt-1 mb-1 pb-s">
			<code class="color ml-3" ><a class="upub-title" href=title_target >μpub</a></code>
			<small class="ml-1 mr-1 hidden-on-tiny" ><a class="clean" href="/web/server" >micro social network, federated</a></small>
			/* TODO kinda jank with the float but whatever, will do for now */
			<input type="submit" class="mr-2 rev" on:click=move |_| set_menu.set(!menu.get()) value="menu" style="float: right" />
		</nav>
		<hr class="sep sticky" />
		<div class="container mt-2 pt-2" >
			<div class="two-col" >
				<div class="col-side sticky pb-s" class:hidden=move || menu.get() >
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
				<div class="col-main" class:w-100=move || menu.get() >
					<Router // TODO maybe set base="/web" ?
						trailing_slash=TrailingSlash::Redirect
						fallback=|| view! { <NotFound /> }
					>
						<main>
								<Routes>
									<Route path="/" view=move || view! { <Redirect path="/web" /> } />
									<Route path="/web" view=Navigable >
										<Route path="" view=move ||
											if auth.present() {
												view! { <Redirect path="home" /> }
											} else {
												view! { <Redirect path="server" /> }
											}
										/>
										<Route path="home" view=move || view! { <TimelinePage name="home" tl=feeds.home /> } />
										<Route path="server" view=move || view! { <TimelinePage name="server" tl=feeds.global /> } />
										<Route path="local" view=move || view! { <TimelinePage name="local" tl=feeds.server /> } />
										<Route path="inbox" view=move || view! { <TimelinePage name="inbox" tl=feeds.private /> } />

										<Route path="about" view=AboutPage />
										<Route path="config" view=move || view! { <ConfigPage setter=set_config /> } />
										<Route path="dev" view=DebugPage />

										<Route path="actors" view=Outlet > // TODO can we avoid this?
											<Route path=":id" view=ActorHeader >
												<Route path="" view=ActorPosts />
												<Route path="following" view=move || view! { <FollowList outgoing=true /> } />
												<Route path="followers" view=move || view! { <FollowList outgoing=false /> } />
											</Route>
											<Route path="" view=NotFound />
										</Route>

										<Route path="objects/:id" view=ObjectPage />
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
				<span class="footer" >"\u{26fc} woven under moonlight  :: "<a class="clean" href="https://git.alemi.dev/upub.git" target="_blank" >src</a>" :: "<a class="clean" href="mailto:abuse@alemi.dev">contact</a>" :: "<a class="clean" href="/web/dev">dev</a>" :: "<a class="clean" href="javascript:window.scrollTo({top:0, behavior:'smooth'})">top</a></span>
			</div>
		</footer>
	}
}

#[component]
fn Navigable() -> impl IntoView {
	let location = use_location();
	let breadcrumb = Signal::derive(move || {
		let path = location.pathname.get();
		let mut path_iter = path.split('/').skip(1);
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
			Some(p) => p.to_string(),
			None => "?".to_string(),
		}
	});
	view! {
		<div class="tl-header w-100 center" >
			<a class="breadcrumb mr-1" href="javascript:history.back()" ><b>"<<"</b></a>
			<b>{crate::NAME}</b>" :: "{breadcrumb}
		</div>
		<Outlet />
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
pub fn Loader(#[prop(optional)] margin: bool) -> impl IntoView {
	view! {
		<div class="center" class:mt-1={margin}>
			<button type="button" disabled>"loading "<span class="dots"></span></button>
		</div>
	}
}
