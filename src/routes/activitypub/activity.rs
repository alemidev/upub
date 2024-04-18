use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter};
use crate::{errors::UpubError, model::{self, addressing::EmbeddedActivity}, server::{auth::AuthIdentity, Context}};
use apb::{ActivityMut, ObjectMut, BaseMut, Node};

use super::{jsonld::LD, JsonLD, TryFetch};

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

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<TryFetch>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let aid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.aid(id.clone())
	};
	match model::addressing::Entity::find_activities()
		.filter(model::activity::Column::Id.eq(&aid))
		.filter(auth.filter_condition())
		.into_model::<EmbeddedActivity>()
		.one(ctx.db())
		.await?
	{
		Some(activity) => Ok(JsonLD(serde_json::Value::from(activity).ld_context())),
		None => if auth.is_local() && query.fetch && !ctx.is_local(&aid) {
			Ok(JsonLD(ap_activity(ctx.fetch().activity(&aid).await?).ld_context()))
		} else {
			Err(UpubError::not_found())
		},
	}
}

