use leptos::*;
use leptos_router::*;
use reqwest::Method;
use crate::prelude::*;

use leptos_use::{storage::use_local_storage, use_cookie, utils::{FromToStringCodec, JsonCodec}};


#[component]
pub fn App() -> impl IntoView {
	let (token, set_token) = use_cookie::<String, FromToStringCodec>("token");
	let (userid, set_userid) = use_cookie::<String, FromToStringCodec>("user_id");
	let (config, set_config, _) = use_local_storage::<crate::Config, JsonCodec>("config");

	if let Some(tok) = token.get_untracked() {
		spawn_local(async move {
			match reqwest::Client::new()
				.request(Method::PATCH, format!("{URL_BASE}/auth"))
				.json(&serde_json::json!({"token": tok}))
				.send()
				.await
			{
				Err(e) => tracing::error!("could not refresh token: {e}"),
				Ok(res) => match res.error_for_status() {
					Err(e) => tracing::error!("server rejected refresh: {e}"),
					Ok(doc) => match doc.json::<AuthResponse>().await {
						Err(e) => tracing::error!("failed parsing auth response: {e}"),
						Ok(auth) => {
							set_token.set(Some(auth.token));
							set_userid.set(Some(auth.user));
						},
					}
				}
			}
		})
	};

	let auth = Auth { token, userid };
	provide_context(auth);
	provide_context(config);

	let username = auth.userid.get_untracked()
		.map(|x| x.split('/').last().unwrap_or_default().to_string())
		.unwrap_or_default();
	let home_tl = Timeline::new(format!("{URL_BASE}/users/{username}/inbox/page"));
	let server_tl = Timeline::new(format!("{URL_BASE}/inbox/page"));
	let user_tl = Timeline::new(format!("{URL_BASE}/users/{username}/outbox/page"));
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

	let auth_present = auth.token.get_untracked().is_some(); // skip helper to use get_untracked
	if auth_present { 
		spawn_local(async move {
			if let Err(e) = home_tl.more(auth).await {
				tracing::error!("error populating timeline: {e}");
			}
		});
	}

	let title_target = if auth_present { "/web/home" } else { "/web/server" };

	view! {
		<nav class="w-100 mt-1 mb-1 pb-s">
			<code class="color ml-3" ><a class="upub-title" href={title_target} >μpub</a></code>
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
						home_tl=home_tl
						server_tl=server_tl
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

											<Route path="/web/about" view=AboutPage />
											<Route path="/web/config" view=move || view! { <ConfigPage setter=set_config /> } />
											<Route path="/web/config/dev" view=DebugPage />

											<Route path="/web/users/:id" view=move || view! { <UserPage tl=user_tl /> } />
											<Route path="/web/objects/:id" view=move || view! { <ObjectPage tl=context_tl /> } />

											<Route path="/web/search" view=SearchPage />
											<Route path="/web/register" view=RegisterPage />

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
			<div class="sep-top">
				<span class="footer" >"\u{26fc} woven under moonlight  :: "<a class="clean" href="https://git.alemi.dev/upub.git" target="_blank" >src</a>" :: "<a class="clean" href="mailto:abuse@alemi.dev">contact</a>" :: "<a class="clean" href="javascript:window.scrollTo({top:0, behavior:'smooth'})">top</a></span>
			</div>
		</footer>
	}
}
