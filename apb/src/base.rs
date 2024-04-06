use crate::{LinkType, ObjectType};

crate::strenum! {
	pub enum BaseType {
		;
		Object(ObjectType),
		Link(LinkType)
	};
}

pub trait Base {
	fn id(&self) -> Option<&str> { None }
	fn base_type(&self) -> Option<BaseType> { None }
}


pub trait BaseMut {
	fn set_id(self, val: Option<&str>) -> Self;
	fn set_base_type(self, val: Option<BaseType>) -> Self;
}


impl Base for String {
	fn id(&self) -> Option<&str> {
		Some(self)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Link(LinkType::Link))
	}
}

#[cfg(feature = "unstructured")]
impl Base for serde_json::Value {
	crate::getter! { id -> &str }
	crate::getter! { base_type -> type BaseType }
}

#[cfg(feature = "unstructured")]
impl BaseMut for serde_json::Value {
	crate::setter! { id -> &str }
	crate::setter! { base_type -> type BaseType }
}
