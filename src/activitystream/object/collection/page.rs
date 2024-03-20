use crate::{activitystream::Node, getter, setter};

pub trait CollectionPage : super::Collection {
	fn part_of(&self) -> Node<impl super::Collection> { Node::Empty::<serde_json::Value> }
	fn next(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn prev(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
}

pub trait CollectionPageMut : super::CollectionMut {
	fn set_part_of(self, val: Node<impl super::Collection>) -> Self;
	fn set_next(self, val: Node<impl CollectionPage>) -> Self;
	fn set_prev(self, val: Node<impl CollectionPage>) -> Self;
}

impl CollectionPage for serde_json::Value {
	getter! { part_of::partOf -> node impl super::Collection }
	getter! { next -> node impl CollectionPage }
	getter! { prev -> node impl CollectionPage }
}

impl CollectionPageMut for serde_json::Value {
	setter! { part_of::partOf -> node impl super::Collection }
	setter! { next -> node impl CollectionPage }
	setter! { prev -> node impl CollectionPage }
}
