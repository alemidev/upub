pub trait Tombstone : super::Object {
	fn former_type(&self) -> Option<super::super::BaseType> { None }
	fn deleted(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
}

pub trait TombstoneMut : super::ObjectMut {
	fn set_former_type(&mut self, val: Option<super::super::BaseType>) -> &mut Self;
	fn set_deleted(&mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> &mut Self;
}

impl Tombstone for serde_json::Value {
	// ... TODO
}
