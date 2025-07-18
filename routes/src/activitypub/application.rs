use apb::{ActorMut, BaseMut, ObjectMut, PublicKeyMut, LD};
use axum::{extract::State, response::{IntoResponse, Response}};
use upub::Context;

use crate::builders::JsonLD;


pub async fn view(State(ctx): State<Context>) -> crate::ApiResult<Response> {
	Ok(JsonLD(
		apb::new()
			.set_id(Some(upub::url!(ctx, "")))
			.set_actor_type(Some(apb::ActorType::Application))
			.set_name(Some(ctx.cfg().instance.name.clone()))
			.set_summary(Some(ctx.cfg().instance.description.clone()))
			.set_inbox(apb::Node::link(upub::url!(ctx, "/inbox")))
			.set_outbox(apb::Node::link(upub::url!(ctx, "/outbox")))
			.set_published(Some(ctx.actor().published))
			.set_endpoints(apb::Node::Empty)
			.set_preferred_username(Some(ctx.domain().to_string()))
			.set_url(apb::Node::link(upub::url!(ctx, "/")))
			.set_public_key(apb::Node::object(
				apb::new()
					.set_id(Some(upub::url!(ctx, "#main-key")))
					.set_owner(Some(upub::url!(ctx, "")))
					.set_public_key_pem(ctx.actor().public_key.clone())
			))
			.ld_context()
	).into_response())
}

