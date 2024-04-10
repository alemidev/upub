pub trait Profile : super::Object {
	// not a Node because it's always embedded and one
	fn describes(&self) -> crate::Node<Self::Object> { crate::Node::Empty }
}

#[cfg(feature = "unstructured")]
impl Profile for serde_json::Value {

}
