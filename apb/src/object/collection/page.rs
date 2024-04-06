use crate::{Node, getter, setter};

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

impl CollectionPage for serde_json::Value {
	getter! { part_of::partOf -> node Self::Collection }
	getter! { next -> node Self::CollectionPage }
	getter! { prev -> node Self::CollectionPage }
}

impl CollectionPageMut for serde_json::Value {
	setter! { part_of::partOf -> node Self::Collection }
	setter! { next -> node Self::CollectionPage }
	setter! { prev -> node Self::CollectionPage }
}
