pub mod page;
pub use page::CollectionPage;

use crate::activitystream::Node;
use crate::{getter, setter, strenum};

strenum! {
	pub enum CollectionType {
		Collection,
		CollectionPage,
		OrderedCollection,
		OrderedCollectionPage;
	};
}

pub trait Collection : super::Object {
	fn collection_type(&self) -> Option<CollectionType> { None }

	fn total_items(&self) -> Option<u64> { None }
	fn current(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn first(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn last(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn items(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn ordered_items(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
}

pub trait CollectionMut : super::ObjectMut {
	fn set_collection_type(&mut self, val: Option<CollectionType>) -> &mut Self;

	fn set_total_items(&mut self, val: Option<u64>) -> &mut Self;
	fn set_current(&mut self, val: Node<impl CollectionPage>) -> &mut Self;
	fn set_first(&mut self, val: Node<impl CollectionPage>) -> &mut Self;
	fn set_last(&mut self, val: Node<impl CollectionPage>) -> &mut Self;
	fn set_items(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_ordered_items(&mut self, val: Node<impl super::Object>) -> &mut Self;
}

impl Collection for serde_json::Value {
	getter! { collection_type -> type CollectionType }
	getter! { total_items::totalItems -> u64 }
	getter! { current -> node impl CollectionPage }
	getter! { first -> node impl CollectionPage }
	getter! { last -> node impl CollectionPage }
	getter! { items -> node impl super::Object }
	getter! { ordered_items::orderedItems -> node impl super::Object }
}

impl CollectionMut for serde_json::Value {
	setter! { collection_type -> type CollectionType }
	setter! { total_items::totalItems -> u64 }
	setter! { current -> node impl CollectionPage }
	setter! { first -> node impl CollectionPage }
	setter! { last -> node impl CollectionPage }
	setter! { items -> node impl super::Object }
	setter! { ordered_items::orderedItems -> node impl super::Object }
}
