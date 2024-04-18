use leptos::*;
use crate::prelude::*;


pub type Auth = Signal<Option<String>>;
pub trait AuthToken {
	fn present(&self) -> bool;
	fn token(&self) -> String;
}


#[component]
pub fn LoginBox(
	token_tx: WriteSignal<Option<String>>,
	token: Signal<Option<String>>,
	username: Signal<Option<String>>,
	username_tx: WriteSignal<Option<String>>,
	home_tl: Timeline,
	server_tl: Timeline,
) -> impl IntoView {
	let username_ref: NodeRef<html::Input> = create_node_ref();
	let password_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div>
			<div class="w-100" class:hidden=move || !token.present() >
				"hi "<a href={move || Uri::web("users", &username.get().unwrap_or_default() )} >{move || username.get().unwrap_or_default() }</a>
				<input style="float:right" type="submit" value="logout" on:click=move |_| {
					token_tx.set(None);
					home_tl.reset(format!("{URL_BASE}/outbox/page"));
					server_tl.reset(format!("{URL_BASE}/inbox/page"));
					spawn_local(async move {
						if let Err(e) = server_tl.more(token).await {
							logging::error!("failed refreshing server timeline: {e}");
						}
					});
				} />
			</div>
			<div class:hidden=move || token.present() >
				<input class="w-100" type="text" node_ref=username_ref placeholder="username" />
				<input class="w-100" type="text" node_ref=password_ref placeholder="password" />
				<input class="w-100" type="submit" value="login" on:click=move |_| {
					logging::log!("logging in...");
					let email = username_ref.get().map(|x| x.value()).unwrap_or("".into());
					let password = password_ref.get().map(|x| x.value()).unwrap_or("".into());
					spawn_local(async move {
						let auth = reqwest::Client::new()
							.post(format!("{URL_BASE}/auth"))
							.json(&LoginForm { email, password })
							.send()
							.await.unwrap()
							.json::<AuthResponse>()
							.await.unwrap();
						logging::log!("logged in until {}", auth.expires);
						// update our username and token cookies
						let username = auth.user.split('/').last().unwrap_or_default().to_string();
						username_tx.set(Some(username.clone()));
						token_tx.set(Some(auth.token));
						// reset home feed and point it to our user's inbox
						home_tl.reset(format!("{URL_BASE}/users/{}/inbox/page", username));
						spawn_local(async move {
							if let Err(e) = home_tl.more(token).await {
								tracing::error!("failed refreshing home timeline: {e}");
							}
						});
						// reset server feed: there may be more content now that we're authed
						server_tl.reset(format!("{URL_BASE}/inbox/page"));
						spawn_local(async move {
							if let Err(e) = server_tl.more(token).await {
								tracing::error!("failed refreshing server timeline: {e}");
							}
						});
					});
				} />
			</div>
		</div>
	}
}


#[derive(Debug, serde::Serialize)]
struct LoginForm {
	email: String,
	password: String,
}


#[derive(Debug, Clone, serde::Deserialize)]
struct AuthResponse {
	token: String,
	user: String,
	expires: chrono::DateTime<chrono::Utc>,
}

impl AuthToken for Signal<Option<String>> {
	fn token(&self) -> String {
		match self.get() {
			None => String::new(),
			Some(x) => x.clone(),
		}
	}
	fn present(&self) -> bool {
		match self.get() {
			None => false,
			Some(x) => !x.is_empty(),
		}
	}
}
