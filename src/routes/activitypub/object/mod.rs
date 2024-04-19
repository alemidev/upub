pub mod replies;

use apb::{BaseMut, CollectionMut, ObjectMut};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, QueryFilter};

use crate::{errors::UpubError, model::{self, addressing::EmbeddedActivity}, server::{auth::AuthIdentity, fetcher::Fetcher, Context}};

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

	let result = model::addressing::Entity::find_objects()
		.filter(model::object::Column::Id.eq(&oid))
		.filter(auth.filter_condition())
		.into_model::<EmbeddedActivity>()
		.one(ctx.db())
		.await?;

	let object = match result {
		Some(EmbeddedActivity { activity: _, object: Some(obj) }) => obj,
		_ => {
			if auth.is_local() && query.fetch && !ctx.is_local(&oid) {
				ctx.fetch_object(&oid).await?
			} else {
				return Err(UpubError::not_found()) 
			}
		},
	};

	let replies = 
		serde_json::Value::new_object()
			.set_id(Some(&crate::url!(ctx, "/objects/{id}/replies")))
			.set_collection_type(Some(apb::CollectionType::OrderedCollection))
			.set_first(apb::Node::link(crate::url!(ctx, "/objects/{id}/replies/page")))
			.set_total_items(Some(object.comments as u64));


	Ok(JsonLD(
		object.ap()
			.set_replies(apb::Node::object(replies))
			.ld_context()
	))
}
