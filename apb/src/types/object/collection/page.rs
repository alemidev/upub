use crate::Node;

pub trait CollectionPage : super::Collection {
	fn part_of(&self) -> Node<Self::Collection> { Node::Empty }
	fn next(&self) -> Node<Self::CollectionPage> { Node::Empty }
	fn prev(&self) -> Node<Self::CollectionPage> { Node::Empty }
}

pub trait CollectionPageMut : super::CollectionMut {
	fn set_part_of(self, val: Node<Self::Collection>) -> Self;
	fn set_next(self, val: Node<Self::CollectionPage>) -> Self;
	fn set_prev(self, val: Node<Self::CollectionPage>) -> Self;
}

#[cfg(feature = "unstructured")]
impl CollectionPage for serde_json::Value {
	crate::getter! { partOf -> node Self::Collection }
	crate::getter! { next -> node Self::CollectionPage }
	crate::getter! { prev -> node Self::CollectionPage }
}

#[cfg(feature = "unstructured")]
impl CollectionPageMut for serde_json::Value {
	crate::setter! { partOf -> node Self::Collection }
	crate::setter! { next -> node Self::CollectionPage }
	crate::setter! { prev -> node Self::CollectionPage }
}
