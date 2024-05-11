use axum::{extract::State, Json};

use crate::server::Context;


pub async fn get(
	State(ctx): State<Context>,
) -> crate::Result<Json<mastodon_async_entities::instance::Instance>> {
	Ok(Json(mastodon_async_entities::instance::Instance {
		uri: ctx.domain().to_string(),
		title: ctx.cfg().instance.name.clone(),
		description: ctx.cfg().instance.description.clone(),
		email: ctx.cfg().instance.contact.as_deref().unwrap_or_default().to_string(),
		version: crate::VERSION.to_string(),
		urls: None,
		stats: None,
		thumbnail: None,
		languages: None,
		contact_account: None,
		max_toot_chars: None,
	}))
}
