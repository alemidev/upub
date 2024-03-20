use crate::activitystream::Node;

pub trait Relationship : super::Object {
	fn subject(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn relationship(&self) -> Option<&str> { None } // TODO what does this mean???
	fn object(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
}

pub trait RelationshipMut : super::ObjectMut {
	fn set_subject(&mut self, val: Node<impl super::Object>) -> &mut Self;
	fn set_relationship(&mut self, val: Option<&str>) -> &mut Self; // TODO what does this mean???
	fn set_object(&mut self, val: Node<impl super::Object>) -> &mut Self;
}

impl Relationship for serde_json::Value {
	// ... TODO
}
