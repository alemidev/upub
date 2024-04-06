use crate::Node;

pub trait Relationship : super::Object {
	fn subject(&self) -> Node<Self::Object> { Node::Empty }
	fn relationship(&self) -> Option<&str> { None } // TODO what does this mean???
	// TODO was just object but clashes with Activity
	fn relationship_object(&self) -> Node<Self::Object> { Node::Empty }
}

pub trait RelationshipMut : super::ObjectMut {
	fn set_subject(self, val: Node<Self::Object>) -> Self;
	fn set_relationship(self, val: Option<&str>) -> Self; // TODO what does this mean???
	// TODO was just object but clashes with Activity
	fn set_relationship_object(self, val: Node<Self::Object>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Relationship for serde_json::Value {
	// ... TODO
}
