use leptos::{leptos_dom::logging::console_error, *};
use leptos_router::*;

use leptos_use::{use_cookie, utils::FromToStringCodec};
use upub_web::{
	URL_BASE, context::Timeline, About, LoginBox, MaybeToken, ObjectPage, PostBox,
	TimelinePage, TimelineNavigation, UserPage
};

fn main() {
	_ = console_log::init_with_level(log::Level::Info);
	console_error_panic_hook::set_once();
	let (cookie, set_cookie) = use_cookie::<String, FromToStringCodec>("token");
	let (username, set_username) = use_cookie::<String, FromToStringCodec>("username");
	provide_context(cookie);

	let home_tl = Timeline::new(format!("{URL_BASE}/users/{}/inbox/page", username.get().unwrap_or_default()));
	let server_tl = Timeline::new(format!("{URL_BASE}/inbox/page"));

	let (menu, set_menu) = create_signal(false);

	spawn_local(async move {
		if let Err(e) = server_tl.more(cookie).await {
			console_error(&format!("error populating timeline: {e}"));
		}
	});

	if cookie.get().is_some() {
		spawn_local(async move {
			if let Err(e) = home_tl.more(cookie).await {
				console_error(&format!("error populating timeline: {e}"));
			}
		});
	}

	mount_to_body(
		move || view! {
			<nav class="w-100 mt-1 mb-1 pb-s">
				<code class="color ml-3" ><a class="upub-title" href=move || if cookie.get().present() { "/web/home" } else { "/web/server" } >μpub</a></code>
				<small class="ml-1 mr-1" ><a class="clean" href="/web/server" >micro social network, federated</a></small>
				/* TODO kinda jank with the float but whatever, will do for now */
				<input type="submit" class="mr-2 rev" on:click=move |_| set_menu.set(!menu.get()) value="menu" style="float: right" />
			</nav>
			<hr class="sep" />
			<div class="container mt-2 pt-2" >
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
								<div class="two-col" >
									<div class="col-side sticky" class:hidden=move || menu.get() >
										<LoginBox
											token_tx=set_cookie
											token=cookie
											username_tx=set_username
											username=username
											home_tl=home_tl
										/>
										<hr class="mt-1 mb-1" />
										<TimelineNavigation />
										<hr class="mt-1 mb-1" />
										<PostBox username=username />
									</div>
									<div class="col-main" class:w-100=move || menu.get() >
										<Routes>
											<Route path="/web" view=About />

											<Route path="/web/home" view=move || view! { <TimelinePage name="home" tl=home_tl /> } />
											<Route path="/web/server" view=move || view! { <TimelinePage name="server" tl=server_tl /> } />

											<Route path="/web/users/:id" view=UserPage />
											<Route path="/web/objects/:id" view=ObjectPage />

											<Route path="/" view=move || view! { <Redirect path="/web" /> } />
										</Routes>
									</div>
								</div>
							</main>
						}
					}}
				</Router>
			</div>
			<footer>
				<div>
					<hr class="sep" />
					<span class="footer" >"\u{26fc} woven under moonlight  :: "<a href="https://git.alemi.dev/upub.git" target="_blank" >src</a>" :: wip by alemi "</span>
				</div>
			</footer>
		}
	);
}
