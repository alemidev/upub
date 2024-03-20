pub trait Profile : super::Object {
	// not a Node because it's always embedded and one
	fn describes(&self) -> Option<impl super::Object> { None::<serde_json::Value> }
}

impl Profile for serde_json::Value {

}
