use apb::{BaseMut, CollectionMut, CollectionPageMut, LD};
use axum::response::{IntoResponse, Response};

use crate::activitypub::Pagination;

pub fn collection_page(id: &str, page: Pagination, items: apb::Node<serde_json::Value>) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	let (limit, offset) = page.pagination();
	let next = if items.len() < limit as usize {
		apb::Node::Empty
	} else {
		apb::Node::link(format!("{id}?offset={}", offset+limit))
	};
	Ok(JsonLD(
		apb::new()
			.set_id(Some(format!("{id}?offset={offset}")))
			.set_collection_type(Some(apb::CollectionType::OrderedCollectionPage))
			.set_part_of(apb::Node::link(id.replace("/page", "")))
			.set_ordered_items(items)
			.set_next(next)
			.ld_context()
	))
}


pub fn collection(id: String, total_items: Option<u64>) -> crate::ApiResult<JsonLD<serde_json::Value>> {
	Ok(JsonLD(
		apb::new()
			.set_collection_type(Some(apb::CollectionType::OrderedCollection))
			.set_first(apb::Node::link(format!("{id}/page")))
			.set_total_items(total_items)
			.set_id(Some(id))
			.ld_context()
	))
}

// got this from https://github.com/kitsune-soc/kitsune/blob/b023a12b687dd9a274233a5a9950f2de5e192344/kitsune/src/http/responder.rs
// i was trying to do it with middlewares but this is way cleaner
pub struct JsonLD<T>(pub T);
impl<T: serde::Serialize> IntoResponse for JsonLD<T> {
	fn into_response(self) -> Response {
		(
			[("Content-Type", apb::jsonld::CONTENT_TYPE_LD_JSON_ACTIVITYPUB)],
			axum::Json(self.0)
		).into_response()
	}
}
