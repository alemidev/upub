use leptos::*;
use crate::prelude::*;


pub trait AuthToken {
	fn present(&self) -> bool;
	fn token(&self) -> String;
	fn username(&self) -> String;
	fn outbox(&self) -> String;
}

#[derive(Debug, Clone, Copy)]
pub struct Auth {
	pub token: Signal<Option<String>>,
	pub user: Signal<Option<String>>,
}


#[component]
pub fn LoginBox(
	token_tx: WriteSignal<Option<String>>,
	username_tx: WriteSignal<Option<String>>,
	home_tl: Timeline,
	server_tl: Timeline,
) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let username_ref: NodeRef<html::Input> = create_node_ref();
	let password_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div>
			<div class="w-100" class:hidden=move || !auth.present() >
				"hi "<a href={move || Uri::web(FetchKind::User, &auth.username() )} >{move || auth.username() }</a>
				<input style="float:right" type="submit" value="logout" on:click=move |_| {
					token_tx.set(None);
					home_tl.reset(format!("{URL_BASE}/outbox/page"));
					server_tl.reset(format!("{URL_BASE}/inbox/page"));
					spawn_local(async move {
						if let Err(e) = server_tl.more(auth).await {
							logging::error!("failed refreshing server timeline: {e}");
						}
					});
				} />
			</div>
			<div class:hidden=move || auth.present() >
				<form on:submit=move|ev| {
					ev.prevent_default();
					logging::log!("logging in...");
					let email = username_ref.get().map(|x| x.value()).unwrap_or("".into());
					let password = password_ref.get().map(|x| x.value()).unwrap_or("".into());
					spawn_local(async move {
						let Ok(res) = reqwest::Client::new()
							.post(format!("{URL_BASE}/auth"))
							.json(&LoginForm { email, password })
							.send()
							.await
						else { if let Some(rf) = password_ref.get() { rf.set_value("") }; return };
						let Ok(auth_response) = res
							.json::<AuthResponse>()
							.await
						else { if let Some(rf) = password_ref.get() { rf.set_value("") }; return };
						logging::log!("logged in until {}", auth_response.expires);
						// update our username and token cookies
						let username = auth_response.user.split('/').last().unwrap_or_default().to_string();
						username_tx.set(Some(username.clone()));
						token_tx.set(Some(auth_response.token));
						// reset home feed and point it to our user's inbox
						home_tl.reset(format!("{URL_BASE}/users/{}/inbox/page", username));
						spawn_local(async move {
							if let Err(e) = home_tl.more(auth).await {
								tracing::error!("failed refreshing home timeline: {e}");
							}
						});
						// reset server feed: there may be more content now that we're authed
						server_tl.reset(format!("{URL_BASE}/inbox/page"));
						spawn_local(async move {
							if let Err(e) = server_tl.more(auth).await {
								tracing::error!("failed refreshing server timeline: {e}");
							}
						});
					});
				} >
					<input class="w-100" type="text" node_ref=username_ref placeholder="username" />
					<input class="w-100" type="password" node_ref=password_ref placeholder="password" />
					<input class="w-100" type="submit" value="login" />
				</form>
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

impl AuthToken for Auth {
	fn token(&self) -> String {
		self.token.get().unwrap_or_default()
	}
	
	fn username(&self) -> String {
		self.user.get().unwrap_or_default()
	}

	fn present(&self) -> bool {
		self.token.get().map_or(false, |x| !x.is_empty())
	}

	fn outbox(&self) -> String {
		format!("{URL_BASE}/users/{}/outbox", self.user.get().unwrap_or_default())
	}
}
