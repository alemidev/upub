use crate::activitystream::Node;

pub trait Relationship : super::Object {
	fn subject(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
	fn relationship(&self) -> Option<&str> { None } // TODO what does this mean???
	// TODO was just object but clashes with Activity
	fn relationship_object(&self) -> Node<impl super::Object> { Node::Empty::<serde_json::Value> }
}

pub trait RelationshipMut : super::ObjectMut {
	fn set_subject(self, val: Node<impl super::Object>) -> Self;
	fn set_relationship(self, val: Option<&str>) -> Self; // TODO what does this mean???
	// TODO was just object but clashes with Activity
	fn set_relationship_object(self, val: Node<impl super::Object>) -> Self;
}

impl Relationship for serde_json::Value {
	// ... TODO
}
