use leptos::*;
use reqwest::Method;
use crate::{components::AuthResponse, URL_BASE};

#[derive(Debug, Clone, Copy)]
pub struct Auth {
	pub token: Signal<Option<String>>,
	pub userid: Signal<Option<String>>,
}

impl Auth {
	pub fn token(&self) -> String {
		self.token.get().unwrap_or_default()
	}

	pub fn user_id(&self) -> String {
		self.userid.get().unwrap_or_default()
	}
	
	pub fn username(&self) -> String {
		// TODO maybe cache this?? how often do i need it?
		self.userid.get()
			.unwrap_or_default()
			.split('/')
			.last()
			.unwrap_or_default()
			.to_string()
	}

	pub fn present(&self) -> bool {
		self.token.get().map_or(false, |x| !x.is_empty())
	}

	pub fn outbox(&self) -> String {
		format!("{URL_BASE}/actors/{}/outbox", self.username())
	}

	pub async fn refresh(
		token: Signal<Option<String>>,
		set_token: WriteSignal<Option<String>>,
		set_userid: WriteSignal<Option<String>>
	) -> bool {
		if let Some(tok) = token.get_untracked() {
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
							return true;
						},
					}
				}
			}
		}
		false
	}
}
