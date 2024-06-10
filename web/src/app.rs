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
						fallback=move || view! { 
							<Breadcrumb back=true >404</Breadcrumb>
							<div class="center">
								<h3>nothing to see here!</h3>
								<p><a href="/web"><button type="button">back to root</button></a></p>
							</div>
						}.into_view()
					>
						// TODO this is kind of ugly: the whole router gets rebuilt every time we log in/out
						// in a sense it's what we want: refreshing the home tl is main purpose, but also
						// server tl may contain stuff we can no longer see, or otherwise we may now be
						// entitled to see new posts. so while being ugly it's techically correct ig?
						<main>
								<Routes>
									<Route path="/web" view=move ||
										if auth.present() {
											view! { <Redirect path="/web/home" /> }
										} else {
											view! { <Redirect path="/web/server" /> }
										}
									/>

									<Route path="/web/home" view=move || view! { <TimelinePage name="home" tl=feeds.home /> } />
									<Route path="/web/server" view=move || view! { <TimelinePage name="server" tl=feeds.global /> } />
									<Route path="/web/local" view=move || view! { <TimelinePage name="local" tl=feeds.server /> } />
									<Route path="/web/inbox" view=move || view! { <TimelinePage name="inbox" tl=feeds.private /> } />

									<Route path="/web/about" view=AboutPage />
									<Route path="/web/config" view=move || view! { <ConfigPage setter=set_config /> } />
									<Route path="/web/dev" view=DebugPage />
									<Route path="/web/config/dev" view=DebugPage />

									<Route path="/web/actors/:id" view=UserPage />
									<Route path="/web/actors/:id/following" view=move || view! { <FollowPage outgoing=true /> } />
									<Route path="/web/actors/:id/followers" view=move || view! { <FollowPage outgoing=false /> } />

									<Route path="/web/objects/:id" view=ObjectPage />
									// <Route path="/web/activities/:id" view=move || view! { <ActivityPage tl=context_tl /> } />

									<Route path="/web/search" view=SearchPage />
									<Route path="/web/register" view=RegisterPage />

									<Route path="/" view=move || view! { <Redirect path="/web" /> } />
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
