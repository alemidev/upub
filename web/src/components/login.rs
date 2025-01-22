use leptos::prelude::*;
use crate::prelude::*;

#[component]
pub fn LoginBox(
	token_tx: WriteSignal<Option<String>>,
	userid_tx: WriteSignal<Option<String>>,
) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let username_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	let password_ref: NodeRef<leptos::html::Input> = NodeRef::new();
	view! {
		<div>
			<div class="w-100" class:hidden=move || !auth.present() >
				"hi "<a href={move || Uri::web(U::Actor, &auth.username() )} >{move || auth.username() }</a>
				<input style="float:right" type="submit" value="logout" on:click=move |_| {
					token_tx.set(None);
					crate::cache::OBJECTS.clear();
					crate::cache::TIMELINES.clear();
					crate::cache::WEBFINGER.clear();
				} />
			</div>
			<div class:hidden=move || auth.present() >
				<form on:submit=move|ev| {
					ev.prevent_default();
					tracing::info!("logging in...");
					let email = username_ref.get().map(|x| x.value()).unwrap_or("".into());
					let password = password_ref.get().map(|x| x.value()).unwrap_or("".into());
					leptos::task::spawn_local(async move {
						let res = match crate::Http::request::<LoginForm>(
							reqwest::Method::POST,
							&format!("{URL_BASE}/auth"),
							Some(&LoginForm { email, password }),
							auth,
						).await {
							Ok(res) => res,
							Err(e) => {
								tracing::warn!("could not login: {e}");
								if let Some(rf) = password_ref.get() {
									rf.set_value("")
								};
								return
							}
						};
						let auth_response = match res.json::<AuthResponse>().await {
							Ok(r) => r,
							Err(e) => {
								tracing::warn!("could not deserialize token response: {e}");
								if let Some(rf) = password_ref.get() {
									rf.set_value("")
								};
								return
							},
						};
						tracing::info!("logged in until {}", auth_response.expires);
						// update our username and token cookies
						userid_tx.set(Some(auth_response.user));
						token_tx.set(Some(auth_response.token));
						// clear caches: we may see things differently now that we're logged in!
						crate::cache::OBJECTS.clear();
						crate::cache::TIMELINES.clear();
						crate::cache::WEBFINGER.clear();
					});
				} >
					<table class="w-100 align">
						<tr>
							<td colspan="2"><input class="w-100" type="text" node_ref=username_ref placeholder="username" /></td>
						</tr>
						<tr>
							<td colspan="2"><input class="w-100" type="password" node_ref=password_ref placeholder="password" /></td>
						</tr>
						<tr>
							<td class="w-50"><input class="w-100" type="submit" value="login" /></td>
							<td class="w-50"><a href="/web/register"><input class="w-100" type="button" value="register" /></a></td>
						</tr>
					</table>
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
pub struct AuthResponse {
	pub token: String,
	pub user: String,
	pub expires: chrono::DateTime<chrono::Utc>,
}
