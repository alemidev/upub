pub mod page;
pub use page::CollectionPage;

use crate::{Field, FieldErr, Node, Object, ObjectMut};

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

	fn collection_type(&self) -> Field<CollectionType> { Err(FieldErr("type")) }

	/// A non-negative integer specifying the total number of objects contained by the logical view of the collection.
	/// This number might not reflect the actual number of items serialized within the Collection object instance. 
	fn total_items(&self) -> Field<u64> { Err(FieldErr("totalItems")) }
	/// In a paged Collection, indicates the page that contains the most recently updated member items. 
	fn current(&self) -> Node<Self::CollectionPage> { Node::Empty }
	/// In a paged Collection, indicates the furthest preceeding page of items in the collection. 
	fn first(&self) -> Node<Self::CollectionPage> { Node::Empty }
	/// In a paged Collection, indicates the furthest proceeding page of the collection.
	fn last(&self) -> Node<Self::CollectionPage> { Node::Empty }
	/// Identifies the items contained in a collection. The items might be ordered or unordered.
	fn items(&self) -> Node<Self::Object> { Node::Empty }
	/// ??????????????? same as items but ordered?? spec just uses it without saying
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
	crate::getter! { totalItems -> u64 }
	crate::getter! { current -> node Self::CollectionPage }
	crate::getter! { first -> node Self::CollectionPage }
	crate::getter! { last -> node Self::CollectionPage }
	crate::getter! { items -> node <Self as Object>::Object }
	crate::getter! { orderedItems -> node <Self as Object>::Object }
}

#[cfg(feature = "unstructured")]
impl CollectionMut for serde_json::Value {
	type CollectionPage = serde_json::Value;

	crate::setter! { collection_type -> type CollectionType }
	crate::setter! { totalItems -> u64 }
	crate::setter! { current -> node Self::CollectionPage }
	crate::setter! { first -> node Self::CollectionPage }
	crate::setter! { last -> node Self::CollectionPage }
	crate::setter! { items -> node <Self as Object>::Object }
	crate::setter! { orderedItems -> node <Self as Object>::Object }
}
