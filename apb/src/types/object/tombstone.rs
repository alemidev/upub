pub trait Tombstone : super::Object {
	fn former_type(&self) -> crate::Field<crate::BaseType> { Err(crate::FieldErr("formerType")) }
	fn deleted(&self) -> crate::Field<chrono::DateTime<chrono::Utc>> { Err(crate::FieldErr("deleted")) }
}

pub trait TombstoneMut : super::ObjectMut {
	fn set_former_type(self, val: Option<crate::BaseType>) -> Self;
	fn set_deleted(self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Tombstone for serde_json::Value {
	// ... TODO
}
