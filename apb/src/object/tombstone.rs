pub trait Tombstone : super::Object {
	fn former_type(&self) -> Option<crate::BaseType> { None }
	fn deleted(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
}

pub trait TombstoneMut : super::ObjectMut {
	fn set_former_type(self, val: Option<crate::BaseType>) -> Self;
	fn set_deleted(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Tombstone for serde_json::Value {
	// ... TODO
}
