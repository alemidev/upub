use leptos::*;
use crate::prelude::*;

#[component]
pub fn LoginBox(
	token_tx: WriteSignal<Option<String>>,
	userid_tx: WriteSignal<Option<String>>,
	home_tl: Timeline,
	server_tl: Timeline,
) -> impl IntoView {
	let auth = use_context::<Auth>().expect("missing auth context");
	let username_ref: NodeRef<html::Input> = create_node_ref();
	let password_ref: NodeRef<html::Input> = create_node_ref();
	view! {
		<div>
			<div class="w-100" class:hidden=move || !auth.present() >
				"hi "<a href={move || Uri::web(U::User, &auth.username() )} >{move || auth.username() }</a>
				<input style="float:right" type="submit" value="logout" on:click=move |_| {
					token_tx.set(None);
					home_tl.reset(format!("{URL_BASE}/outbox/page"));
					server_tl.reset(format!("{URL_BASE}/inbox/page"));
					server_tl.more(auth);
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
						userid_tx.set(Some(auth_response.user));
						token_tx.set(Some(auth_response.token));
						// reset home feed and point it to our user's inbox
						home_tl.reset(format!("{URL_BASE}/actors/{}/inbox/page", username));
						home_tl.more(auth);
						// reset server feed: there may be more content now that we're authed
						server_tl.reset(format!("{URL_BASE}/inbox/page"));
						server_tl.more(auth);
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
