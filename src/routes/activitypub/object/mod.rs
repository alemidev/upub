pub mod replies;

use apb::{CollectionMut, ObjectMut};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};

use crate::{errors::UpubError, model::{self, addressing::Event}, server::{auth::AuthIdentity, fetcher::Fetcher, Context}};

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
		let obj = ctx.fetch_object(&oid).await?;
		// some implementations serve statuses on different urls than their AP id
		if obj.id != oid {
			return Err(UpubError::Redirect(crate::url!(ctx, "/objects/{}", ctx.id(&obj.id))));
		}
	}

	let item = model::addressing::Entity::find_addressed(auth.my_id())
		.filter(model::object::Column::Id.eq(&oid))
		.filter(auth.filter_condition())
		.into_model::<Event>()
		.one(ctx.db())
		.await?
		.ok_or_else(UpubError::not_found)?;

	let (object, liked) = match item {
		Event::Tombstone => return Err(UpubError::not_found()),
		Event::Activity(_) => return Err(UpubError::not_found()),
		Event::StrayObject { object, liked } => (object, liked),
		Event::DeepActivity { activity: _, liked, object } => (object, liked),
	};

	let attachments = object.find_related(model::attachment::Entity)
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|x| x.ap())
		.collect::<Vec<serde_json::Value>>();

	// let replies = 
	// 	serde_json::Value::new_object()
	// 		.set_id(Some(&crate::url!(ctx, "/objects/{id}/replies")))
	// 		.set_collection_type(Some(apb::CollectionType::OrderedCollection))
	// 		.set_first(apb::Node::link(crate::url!(ctx, "/objects/{id}/replies/page")))
	// 		.set_total_items(Some(object.comments as u64));
	
	let likes_count = object.likes as u64;
	let mut obj = object.ap().set_attachment(apb::Node::array(attachments));

	if let Some(liked) = liked {
		obj = obj.set_audience(apb::Node::object( // TODO setting this again ewww...
			serde_json::Value::new_object()
				.set_collection_type(Some(apb::CollectionType::OrderedCollection))
				.set_total_items(Some(likes_count))
				.set_ordered_items(apb::Node::links(vec![liked]))
		));
	}

	Ok(JsonLD(obj.ld_context()))
}
