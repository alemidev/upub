use axum::{extract::{Path, State}, http::StatusCode};
use sea_orm::EntityTrait;
use crate::{model::{self, activity, object}, server::Context};
use apb::{ActivityMut, ObjectMut, BaseMut, Node};

use super::{jsonld::LD, JsonLD};

// TODO this is used outside /routes, maybe move in model?
pub fn ap_activity(activity: model::activity::Model) -> serde_json::Value {
	serde_json::Value::new_object()
		.set_id(Some(&activity.id))
		.set_activity_type(Some(activity.activity_type))
		.set_actor(Node::link(activity.actor))
		.set_object(Node::maybe_link(activity.object))
		.set_target(Node::maybe_link(activity.target))
		.set_published(Some(activity.published))
		.set_to(Node::links(activity.to.0.clone()))
		.set_bto(Node::Empty)
		.set_cc(Node::links(activity.cc.0.clone()))
		.set_bcc(Node::Empty)
}

pub async fn view(State(ctx) : State<Context>, Path(id): Path<String>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match activity::Entity::find_by_id(ctx.aid(id))
		.find_also_related(object::Entity)
		.one(ctx.db())
		.await
	{
		Ok(Some((activity, Some(object)))) => Ok(JsonLD(
			ap_activity(activity)
				.set_object(Node::object(super::object::ap_object(object)))
				.ld_context()
		)),
		Ok(Some((activity, None))) => Ok(JsonLD(
			ap_activity(activity).ld_context()
		)),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for activity: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}

