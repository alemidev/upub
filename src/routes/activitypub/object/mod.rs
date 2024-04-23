pub mod replies;

use apb::{BaseMut, CollectionMut, ObjectMut};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};

use crate::{errors::UpubError, model::{self, addressing::WrappedObject}, server::{auth::AuthIdentity, fetcher::Fetcher, Context}};

use super::{jsonld::LD, JsonLD, TryFetch};

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<TryFetch>,
) -> crate::Result<JsonLD<serde_json::Value>> {
	let oid = if id.starts_with('+') {
		format!("https://{}", id.replacen('+', "", 1).replace('@', "/"))
	} else {
		ctx.oid(id.clone())
	};
	if auth.is_local() && query.fetch && !ctx.is_local(&oid) {
		ctx.fetch_object(&oid).await?;
	}

	let Some(object) = model::addressing::Entity::find_objects()
		.filter(model::object::Column::Id.eq(&oid))
		.filter(auth.filter_condition())
		.into_model::<WrappedObject>()
		.one(ctx.db())
		.await?
	else {
		return Err(UpubError::not_found());
	};

	let attachments = object.object.find_related(model::attachment::Entity)
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|x| x.ap())
		.collect::<Vec<serde_json::Value>>();

	let replies = 
		serde_json::Value::new_object()
			.set_id(Some(&crate::url!(ctx, "/objects/{id}/replies")))
			.set_collection_type(Some(apb::CollectionType::OrderedCollection))
			.set_first(apb::Node::link(crate::url!(ctx, "/objects/{id}/replies/page")))
			.set_total_items(Some(object.object.comments as u64));

	Ok(JsonLD(
		object.object.ap()
			.set_replies(apb::Node::object(replies))
			.set_attachment(apb::Node::array(attachments))
			.ld_context()
	))
}
