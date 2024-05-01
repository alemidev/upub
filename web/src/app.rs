use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use leptos_use::{use_cookie, use_cookie_with_options, utils::FromToStringCodec, UseCookieOptions};


#[component]
pub fn App() -> impl IntoView {
	let (token, set_token) = use_cookie_with_options::<String, FromToStringCodec>(
		"token",
		UseCookieOptions::default()
			.max_age(1000 * 60 * 60 * 6)
	);
	let (user, set_username) = use_cookie::<String, FromToStringCodec>("username");

	let auth = Auth { token, user };
	provide_context(auth);

	let home_tl = Timeline::new(format!("{URL_BASE}/users/{}/inbox/page", auth.username()));
	let server_tl = Timeline::new(format!("{URL_BASE}/inbox/page"));
	let user_tl = Timeline::new(format!("{URL_BASE}/users/{}/outbox/page", auth.username()));
	let context_tl = Timeline::new(format!("{URL_BASE}/outbox/page"));

	let reply_controls = ReplyControls::default();
	provide_context(reply_controls);

	let screen_width = window().screen().map(|x| x.avail_width().unwrap_or_default()).unwrap_or_default();

	let (menu, set_menu) = create_signal(screen_width <= 786);
	let (advanced, set_advanced) = create_signal(false);

	spawn_local(async move {
		if let Err(e) = server_tl.more(auth).await {
			tracing::error!("error populating timeline: {e}");
		}
	});

	if auth.present() {
		spawn_local(async move {
			if let Err(e) = home_tl.more(auth).await {
				tracing::error!("error populating timeline: {e}");
			}
		});
	}

	let title_target = if auth.present() { "/web/home" } else { "/web/server" };

	view! {
		<nav class="w-100 mt-1 mb-1 pb-s">
			<code class="color ml-3" ><a class="upub-title" href={title_target} >μpub</a></code>
			<small class="ml-1 mr-1 hidden-on-tiny" ><a class="clean" href={title_target} >micro social network, federated</a></small>
			/* TODO kinda jank with the float but whatever, will do for now */
			<input type="submit" class="mr-2 rev" on:click=move |_| set_menu.set(!menu.get()) value="menu" style="float: right" />
		</nav>
		<hr class="sep" />
		<div class="container mt-2 pt-2" >
			<div class="two-col" >
				<div class="col-side sticky pb-s" class:hidden=move || menu.get() >
					<LoginBox
						token_tx=set_token
						username_tx=set_username
						home_tl=home_tl
						server_tl=server_tl
					/>
					<hr class="mt-1 mb-1" />
					<Navigator />
					<hr class="mt-1 mb-1" />
					{move || if advanced.get() { view! {
						<AdvancedPostBox advanced=set_advanced/>
					}} else { view! {
						<PostBox advanced=set_advanced/>
					}}}
				</div>
				<div class="col-main" class:w-100=move || menu.get() >
					<Router // TODO maybe set base="/web" ?
						trailing_slash=TrailingSlash::Redirect
						fallback=move || view! { 
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
						{move || {
							view! {
								<main>
										<Routes>
											<Route path="/web" view=move ||
												if auth.present() {
													view! { <Redirect path="/web/home" /> }
												} else {
													view! { <Redirect path="/web/server" /> }
												}
											/>

											<Route path="/web/home" view=move || view! { <TimelinePage name="home" tl=home_tl /> } />
											<Route path="/web/server" view=move || view! { <TimelinePage name="server" tl=server_tl /> } />

											<Route path="/web/config" view=ConfigPage />
											<Route path="/web/about" view=AboutPage />

											<Route path="/web/users/:id" view=move || view! { <UserPage tl=user_tl /> } />
											<Route path="/web/objects/:id" view=move || view! { <ObjectPage tl=context_tl /> } />

											<Route path="/web/debug" view=DebugPage />

											<Route path="/" view=move || view! { <Redirect path="/web" /> } />
										</Routes>
								</main>
							}
						}}
					</Router>
				</div>
			</div>
		</div>
		<footer>
			<div>
				<hr class="sep" />
				<span class="footer" >"\u{26fc} woven under moonlight  :: "<a href="https://git.alemi.dev/upub.git" target="_blank" >src</a>" :: wip by alemi :: "<a href="javascript:window.scrollTo({top:0, behavior:'smooth'})">top</a></span>
			</div>
		</footer>
	}
}
