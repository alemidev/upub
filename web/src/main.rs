use leptos::{leptos_dom::logging::console_error, *};
use leptos_router::*;

use leptos_use::{use_cookie, utils::JsonCodec};
use upub_web::{
	URL_BASE, context::Timeline, About, Auth, LoginBox, MaybeToken, ObjectPage, PostBox, TimelineFeed, UserPage
};

fn main() {
	_ = console_log::init_with_level(log::Level::Debug);
	console_error_panic_hook::set_once();
	let (cookie, set_cookie) = use_cookie::<Auth, JsonCodec>("token");
	provide_context(cookie);

	let home_tl = Timeline::new(format!("{URL_BASE}/users/{}/inbox/page", cookie.get().username()));
	let server_tl = Timeline::new(format!("{URL_BASE}/inbox/page"));

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
			<nav class="w-100">
				<p>
					<code><a class="upub-title" href="/web/home" >μpub</a></code>
					<small class="ml-1 mr-1" ><a class="clean" href="/web/server" >micro social network, federated</a></small>
					/* TODO kinda jank with the float but whatever, will do for now */
					<small style="float: right" ><a href="https://git.alemi.dev/upub.git" >src</a></small>
				</p>
			</nav>
			<hr />
			<div class="container" >
				<Router // TODO maybe set base="/web" ?
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
									<div class="col-side" >
										<LoginBox
											tx=set_cookie
											rx=cookie
										/>
										<hr class="mt-1 mb-1" />
										<a href="/web/home" >
											<input class="w-100"
												type="submit"
												class:hidden=move || !cookie.get().present()
												class:active=move || use_location().pathname.get().ends_with("/home")
												value="home timeline"
											/>
										</a>
										<a href="/web/server" >
											<input
												class="w-100"
												class:active=move || use_location().pathname.get().ends_with("/server")
												type="submit"
												value="server timeline"
											/>
										</a>
										<hr class="mt-1 mb-1" />
										<PostBox />
									</div>
									<div class="col-main" >
										<Routes>
											<Route path="/web" view=About />

											<Route path="/web/home" view=move || view! { <TimelineFeed name="home" tl=home_tl /> } />
											<Route path="/web/server" view=move || view! { <TimelineFeed name="server" tl=server_tl /> } />

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
		}
	);
}
