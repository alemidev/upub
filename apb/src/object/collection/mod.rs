pub mod page;
pub use page::CollectionPage;

use crate::{Node, Object, object::ObjectMut};

crate::strenum! {
	pub enum CollectionType {
		Collection,
		CollectionPage,
		OrderedCollection,
		OrderedCollectionPage;
	};
}

pub trait Collection : Object {
	type CollectionPage : CollectionPage;

	fn collection_type(&self) -> Option<CollectionType> { None }

	fn total_items(&self) -> Option<u64> { None }
	fn current(&self) -> Node<Self::CollectionPage> { Node::Empty }
	fn first(&self) -> Node<Self::CollectionPage> { Node::Empty }
	fn last(&self) -> Node<Self::CollectionPage> { Node::Empty }
	fn items(&self) -> Node<Self::Object> { Node::Empty }
	fn ordered_items(&self) -> Node<Self::Object> { Node::Empty }
}

pub trait CollectionMut : ObjectMut {
	type CollectionPage : CollectionPage;

	fn set_collection_type(self, val: Option<CollectionType>) -> Self;
	fn set_total_items(self, val: Option<u64>) -> Self;
	fn set_current(self, val: Node<Self::CollectionPage>) -> Self;
	fn set_first(self, val: Node<Self::CollectionPage>) -> Self;
	fn set_last(self, val: Node<Self::CollectionPage>) -> Self;
	fn set_items(self, val: Node<Self::Object>) -> Self;
	fn set_ordered_items(self, val: Node<Self::Object>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Collection for serde_json::Value {
	type CollectionPage = serde_json::Value;

	crate::getter! { collection_type -> type CollectionType }
	crate::getter! { total_items::totalItems -> u64 }
	crate::getter! { current -> node Self::CollectionPage }
	crate::getter! { first -> node Self::CollectionPage }
	crate::getter! { last -> node Self::CollectionPage }
	crate::getter! { items -> node <Self as Object>::Object }
	crate::getter! { ordered_items::orderedItems -> node <Self as Object>::Object }
}

#[cfg(feature = "unstructured")]
impl CollectionMut for serde_json::Value {
	type CollectionPage = serde_json::Value;

	crate::setter! { collection_type -> type CollectionType }
	crate::setter! { total_items::totalItems -> u64 }
	crate::setter! { current -> node Self::CollectionPage }
	crate::setter! { first -> node Self::CollectionPage }
	crate::setter! { last -> node Self::CollectionPage }
	crate::setter! { items -> node <Self as Object>::Object }
	crate::setter! { ordered_items::orderedItems -> node <Self as Object>::Object }
}
