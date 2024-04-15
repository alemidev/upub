use axum::{extract::{Path, State}, http::StatusCode};
use sea_orm::{ColumnTrait, QueryFilter};

use apb::{ObjectMut, BaseMut, Node};
use crate::{model::{self, addressing::EmbeddedActivity}, server::{auth::AuthIdentity, Context}};

use super::{jsonld::LD, JsonLD};

// TODO this is used outside /routes, maybe move in model?
pub fn ap_object(object: model::object::Model) -> serde_json::Value {
	serde_json::Value::new_object()
		.set_id(Some(&object.id))
		.set_object_type(Some(object.object_type))
		.set_attributed_to(Node::maybe_link(object.attributed_to))
		.set_name(object.name.as_deref())
		.set_summary(object.summary.as_deref())
		.set_content(object.content.as_deref())
		.set_context(Node::maybe_link(object.context.clone()))
		.set_published(Some(object.published))
		.set_to(Node::links(object.to.0.clone()))
		.set_bto(Node::Empty)
		.set_cc(Node::links(object.cc.0.clone()))
		.set_bcc(Node::Empty)
}

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	let oid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.oid(id.clone())
	};
	match model::addressing::Entity::find_activities()
		.filter(model::object::Column::Id.eq(oid))
		.filter(auth.filter_condition())
		.into_model::<EmbeddedActivity>()
		.one(ctx.db())
		.await
	{
		Ok(Some(EmbeddedActivity { activity: _, object: Some(object) })) => Ok(JsonLD(ap_object(object).ld_context())),
		Ok(Some(EmbeddedActivity { activity: _, object: None })) => Err(StatusCode::NOT_FOUND),
		Ok(None) => Err(StatusCode::NOT_FOUND),
		Err(e) => {
			tracing::error!("error querying for object: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		},
	}
}
