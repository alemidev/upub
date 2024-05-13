use leptos::*;
use crate::URL_BASE;

pub trait AuthToken {
	fn present(&self) -> bool;
	fn token(&self) -> String;
	fn user_id(&self) -> String;
	fn username(&self) -> String;
	fn outbox(&self) -> String;
}

#[derive(Debug, Clone, Copy)]
pub struct Auth {
	pub token: Signal<Option<String>>,
	pub userid: Signal<Option<String>>,
}

impl AuthToken for Auth {
	fn token(&self) -> String {
		self.token.get().unwrap_or_default()
	}

	fn user_id(&self) -> String {
		self.userid.get().unwrap_or_default()
	}
	
	fn username(&self) -> String {
		// TODO maybe cache this?? how often do i need it?
		self.userid.get()
			.unwrap_or_default()
			.split('/')
			.last()
			.unwrap_or_default()
			.to_string()
	}

	fn present(&self) -> bool {
		self.token.get().map_or(false, |x| !x.is_empty())
	}

	fn outbox(&self) -> String {
		format!("{URL_BASE}/users/{}/outbox", self.username())
	}
}
