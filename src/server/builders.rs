use apb::{BaseMut, CollectionMut, CollectionPageMut};
use sea_orm::{Condition, DatabaseConnection, QueryFilter, QuerySelect, RelationTrait};

use crate::{model::{addressing::Event, attachment::BatchFillable}, routes::activitypub::{jsonld::LD, JsonLD, Pagination}};

pub async fn paginate(
	id: String,
	filter: Condition,
	db: &DatabaseConnection,
	page: Pagination,
	my_id: Option<i64>,
	with_users: bool, // TODO ewww too many arguments for this weird function...
) -> crate::Result<JsonLD<serde_json::Value>> {
	let limit = page.batch.unwrap_or(20).min(50);
	let offset = page.offset.unwrap_or(0);

	let mut select = crate::model::addressing::Entity::find_addressed(my_id);

	if with_users {
		select = select
			.join(sea_orm::JoinType::InnerJoin, crate::model::activity::Relation::Actors.def());
	}

	let items = select
		.filter(filter)
		// TODO also limit to only local activities
		.limit(limit)
		.offset(offset)
		.into_model::<Event>()
		.all(db)
		.await?;

	let mut attachments = items.load_attachments_batch(db).await?;

	let items : Vec<serde_json::Value> = items
		.into_iter()
		.map(|item| {
			let attach = attachments.remove(&item.internal());
			item.ap(attach)
		})
		.collect();

	collection_page(&id, offset, limit, items)
}

pub fn collection_page(id: &str, offset: u64, limit: u64, items: Vec<serde_json::Value>) -> crate::Result<JsonLD<serde_json::Value>> {
	let next = if items.len() < limit as usize {
		apb::Node::Empty
	} else {
		apb::Node::link(format!("{id}?offset={}", offset+limit))
	};
	Ok(JsonLD(
		serde_json::Value::new_object()
			.set_id(Some(&format!("{id}?offset={offset}")))
			.set_collection_type(Some(apb::CollectionType::OrderedCollectionPage))
			.set_part_of(apb::Node::link(id.replace("/page", "")))
			.set_ordered_items(apb::Node::array(items))
			.set_next(next)
			.ld_context()
	))
}


pub fn collection(id: &str, total_items: Option<u64>) -> crate::Result<JsonLD<serde_json::Value>> {
	Ok(JsonLD(
		serde_json::Value::new_object()
			.set_id(Some(id))
			.set_collection_type(Some(apb::CollectionType::OrderedCollection))
			.set_first(apb::Node::link(format!("{id}/page")))
			.set_total_items(total_items)
			.ld_context()
	))
}

#[axum::async_trait]
pub trait AnyQuery {
	async fn any(self, db: &sea_orm::DatabaseConnection) -> crate::Result<bool>;
}

#[axum::async_trait]
impl<T : sea_orm::EntityTrait> AnyQuery for sea_orm::Select<T> {
	async fn any(self, db: &sea_orm::DatabaseConnection) ->	crate::Result<bool> {
		Ok(self.one(db).await?.is_some())
	}
}

#[axum::async_trait]
impl<T : sea_orm::SelectorTrait + Send> AnyQuery for sea_orm::Selector<T> {
	async fn any(self, db: &sea_orm::DatabaseConnection) ->	crate::Result<bool> {
		Ok(self.one(db).await?.is_some())
	}
}
