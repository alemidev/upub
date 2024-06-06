pub mod replies;

use apb::{CollectionMut, ObjectMut, LD};
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter, QuerySelect, SelectColumns};
use upub::{model::{self, addressing::Event}, traits::Fetcher, Context};

use crate::{builders::JsonLD, AuthIdentity};

use super::TryFetch;

pub async fn view(
	State(ctx): State<Context>,
	Path(id): Path<String>,
	AuthIdentity(auth): AuthIdentity,
	Query(query): Query<TryFetch>,
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let oid = ctx.oid(&id);
	if auth.is_local() && query.fetch && !ctx.is_local(&oid) {
		let obj = ctx.fetch_object(&oid).await?;
		// some implementations serve statuses on different urls than their AP id
		if obj.id != oid {
			return Err(crate::ApiError::Redirect(upub::url!(ctx, "/objects/{}", ctx.id(&obj.id))));
		}
	}

	let item = model::addressing::Entity::find_addressed(auth.my_id())
		.filter(model::object::Column::Id.eq(&oid))
		.filter(auth.filter_condition())
		.into_model::<Event>()
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::not_found)?;

	let object = match item {
		Event::Tombstone => return Err(crate::ApiError::not_found()),
		Event::Activity(_) => return Err(crate::ApiError::not_found()),
		Event::StrayObject { liked: _, object } => object,
		Event::DeepActivity { activity: _, liked: _, object } => object,
	};

	let attachments = object.find_related(model::attachment::Entity)
		.all(ctx.db())
		.await?
		.into_iter()
		.map(|x| x.ap())
		.collect::<Vec<serde_json::Value>>();

	let mut replies = apb::Node::Empty;
	
	if ctx.cfg().security.show_reply_ids {
		let replies_ids = model::addressing::Entity::find_addressed(None)
			.filter(model::object::Column::InReplyTo.eq(oid))
			.filter(auth.filter_condition())
			.select_only()
			.select_column(model::object::Column::Id)
			.into_tuple::<String>()
			.all(ctx.db())
			.await?;

		replies = apb::Node::object(
			apb::new()
				// .set_id(Some(&upub::url!(ctx, "/objects/{id}/replies")))
				// .set_first(apb::Node::link(upub::url!(ctx, "/objects/{id}/replies/page")))
				.set_collection_type(Some(apb::CollectionType::Collection))
				.set_total_items(Some(object.replies as u64))
				.set_items(apb::Node::links(replies_ids))
		);
	}
	
	Ok(JsonLD(
		object.ap()
			.set_attachment(apb::Node::array(attachments))
			.set_replies(replies)
			.ld_context()
	))
}
