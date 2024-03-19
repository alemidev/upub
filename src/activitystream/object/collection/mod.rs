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

impl Collection for serde_json::Value {

}

impl CollectionPage for serde_json::Value {

}
