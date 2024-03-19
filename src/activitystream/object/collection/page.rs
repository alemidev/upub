use crate::activitystream::Node;

pub trait CollectionPage : super::Collection {
	fn part_of(&self) -> Node<impl super::Collection> { Node::Empty::<serde_json::Value> }
	fn next(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
	fn prev(&self) -> Node<impl CollectionPage> { Node::Empty::<serde_json::Value> }
}
