use apb::{ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::State, http::HeaderMap, response::{IntoResponse, Redirect, Response}};

use crate::{server::Context, url};

use super::{jsonld::LD, JsonLD};


pub async fn view(
	headers: HeaderMap,
	State(ctx): State<Context>,
) -> crate::Result<Response> {
	if let Some(accept) = headers.get("Accept") {
		if let Ok(accept) = accept.to_str() {
			if accept.contains("text/html") {
				return Ok(Redirect::to("/web").into_response());
			}
		}
	}
	Ok(JsonLD(
		serde_json::Value::new_object()
			.set_id(Some(&url!(ctx, "")))
			.set_actor_type(Some(apb::ActorType::Application))
			.set_name(Some("μpub"))
			.set_summary(Some("micro social network, federated"))
			.set_inbox(apb::Node::link(url!(ctx, "/inbox")))
			.set_outbox(apb::Node::link(url!(ctx, "/outbox")))
			.set_published(Some(ctx.app().created))
			.set_public_key(apb::Node::object(
				serde_json::Value::new_object()
					.set_id(Some(&url!(ctx, "#main-key")))
					.set_owner(Some(&url!(ctx, "")))
					.set_public_key_pem(&ctx.app().public_key)
			))
			.ld_context()
	).into_response())
}
