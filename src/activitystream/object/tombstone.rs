pub trait Tombstone : super::Object {
	fn former_type(&self) -> Option<super::super::BaseType> { None }
	fn deleted(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
}

impl Tombstone for serde_json::Value {
	// ... TODO
}
