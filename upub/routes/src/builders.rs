use apb::{BaseMut, CollectionMut, CollectionPageMut, LD};
use sea_orm::{Condition, ConnectionTrait, QueryFilter, QuerySelect, RelationTrait};
use axum::response::{IntoResponse, Response};
use upub::selector::{BatchFillable, RichActivity, RichObject};

use crate::activitypub::Pagination;

pub async fn paginate_activities(
	id: String,
	filter: Condition,
	db: &impl ConnectionTrait,
	page: Pagination,
	my_id: Option<i64>,
	with_users: bool, // TODO ewww too many arguments for this weird function...
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let mut select = upub::Query::activities(my_id);

	if with_users {
		select = select
			.join(sea_orm::JoinType::InnerJoin, upub::model::activity::Relation::Actors.def());
	}

	let items = select
		.filter(filter)
		// TODO also limit to only local activities
		.limit(limit)
		.offset(offset)
		.into_model::<RichActivity>()
		.all(db)
		.await?
		.with_attachments(db)
		.await?;

	let items : Vec<serde_json::Value> = items
		.into_iter()
		.map(|item| item.ap())
		.collect();

	collection_page(&id, offset, limit, items)
}

// TODO can we merge these two??? there are basically only two differences

pub async fn paginate_objects(
	id: String,
	filter: Condition,
	db: &impl ConnectionTrait,
	page: Pagination,
	my_id: Option<i64>,
	with_users: bool, // TODO ewww too many arguments for this weird function...
) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let mut select = upub::Query::objects(my_id); // <--- difference one

	if with_users {
		select = select
			.join(sea_orm::JoinType::InnerJoin, upub::model::activity::Relation::Actors.def());
	}

	let items = select
		.filter(filter)
		// TODO also limit to only local activities
		.limit(limit)
		.offset(offset)
		.into_model::<RichObject>() // <--- difference two
		.all(db)
		.await?
		.with_attachments(db)
		.await?;

	let items : Vec<serde_json::Value> = items
		.into_iter()
		.map(|item| item.ap())
		.collect();

	collection_page(&id, offset, limit, items)
}

pub fn collection_page(id: &str, offset: u64, limit: u64, items: Vec<serde_json::Value>) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let next = if items.len() < limit as usize {
		apb::Node::Empty
	} else {
		apb::Node::link(format!("{id}?offset={}", offset+limit))
	};
	Ok(JsonLD(
		apb::new()
			.set_id(Some(&format!("{id}?offset={offset}")))
			.set_collection_type(Some(apb::CollectionType::OrderedCollectionPage))
			.set_part_of(apb::Node::link(id.replace("/page", "")))
			.set_ordered_items(apb::Node::array(items))
			.set_next(next)
			.ld_context()
	))
}


pub fn collection(id: &str, total_items: Option<u64>) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	Ok(JsonLD(
		apb::new()
			.set_id(Some(id))
			.set_collection_type(Some(apb::CollectionType::OrderedCollection))
			.set_first(apb::Node::link(format!("{id}/page")))
			.set_total_items(total_items)
			.ld_context()
	))
}

// got this from https://github.com/kitsune-soc/kitsune/blob/b023a12b687dd9a274233a5a9950f2de5e192344/kitsune/src/http/responder.rs
// i was trying to do it with middlewares but this is way cleaner
pub struct JsonLD<T>(pub T);
impl<T: serde::Serialize> IntoResponse for JsonLD<T> {
	fn into_response(self) -> Response {
		(
			[("Content-Type", "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")],
			axum::Json(self.0)
		).into_response()
	}
}
