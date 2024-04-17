use leptos::*;
use leptos_router::*;
use crate::prelude::*;

use leptos_use::{use_cookie, utils::FromToStringCodec};


#[component]
pub fn App() -> impl IntoView {
	let (auth, set_auth) = use_cookie::<String, FromToStringCodec>("token");
	let (username, set_username) = use_cookie::<String, FromToStringCodec>("username");
	provide_context(auth);

	let home_tl = Timeline::new(format!("{URL_BASE}/users/{}/inbox/page", username.get().unwrap_or_default()));
	let server_tl = Timeline::new(format!("{URL_BASE}/inbox/page"));

	let screen_width = window().screen().map(|x| x.avail_width().unwrap_or_default()).unwrap_or_default();

	let (menu, set_menu) = create_signal(screen_width <= 786);

	spawn_local(async move {
		if let Err(e) = server_tl.more(auth).await {
			tracing::error!("error populating timeline: {e}");
		}
	});

	if auth.get().is_some() {
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
				<div class="col-side sticky" class:hidden=move || menu.get() >
					<LoginBox
						token_tx=set_auth
						token=auth
						username_tx=set_username
						username=username
						home_tl=home_tl
						server_tl=server_tl
					/>
					<hr class="mt-1 mb-1" />
					<Navigator />
					<hr class="mt-1 mb-1" />
					<PostBox username=username />
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
												if auth.get().is_some() {
													view! { <Redirect path="/web/home" /> }
												} else {
													view! { <Redirect path="/web/server" /> }
												}
											/>

											<Route path="/web/home" view=move || view! { <TimelinePage name="home" tl=home_tl /> } />
											<Route path="/web/server" view=move || view! { <TimelinePage name="server" tl=server_tl /> } />

											<Route path="/web/about" view=AboutPage />

											<Route path="/web/users/:id" view=UserPage />
											<Route path="/web/objects/:id" view=ObjectPage />

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
				<span class="footer" >"\u{26fc} woven under moonlight  :: "<a href="https://git.alemi.dev/upub.git" target="_blank" >src</a>" :: wip by alemi :: "<a href="javascript:window.scrollTo({top:0})">top</a></span>
			</div>
		</footer>
	}
}
