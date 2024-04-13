use apb::{ActorMut, BaseMut, ObjectMut, PublicKeyMut};
use axum::{extract::State, http::StatusCode};

use crate::{server::Context, url};

use super::{jsonld::LD, JsonLD};


pub async fn view(State(ctx): State<Context>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
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
	))
}
