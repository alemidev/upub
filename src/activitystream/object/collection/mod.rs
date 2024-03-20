pub mod page;
pub use page::CollectionPage;

use crate::activitystream::Node;
use crate::strenum;

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

	fn total_items(&self) -> Option<u32> { None }
	fn current(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn first(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn last(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn items(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
}

pub trait CollectionMut : super::ObjectMut {
	fn set_collection_type(&mut self, val: Option<CollectionType>) -> &mut Self;

	fn set_total_items(&mut self, val: Option<u32>) -> &mut Self;
	fn set_current(&mut self, val: Node<impl CollectionPage>) -> &mut Self;
	fn set_first(&mut self, val: Node<impl CollectionPage>) -> &mut Self;
	fn set_last(&mut self, val: Node<impl CollectionPage>) -> &mut Self;
	fn set_items(&mut self, val: Node<impl super::Object>) -> &mut Self;
}

impl Collection for serde_json::Value {
	// ... TODO
}
