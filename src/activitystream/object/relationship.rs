use crate::activitystream::Node;

pub trait Relationship : super::Object {
	fn subject(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn relationship(&self) -> Option<&str> { None } // TODO what does this mean???
	fn object(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
}

impl Relationship for serde_json::Value {
	// ... TODO
}
