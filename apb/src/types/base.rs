use crate::{LinkType, ObjectType};

crate::strenum! {
	pub enum BaseType {
		;
		Object(ObjectType),
		Link(LinkType)
	};
}

pub trait Base : crate::macros::MaybeSend {
	fn id(&self) -> crate::Field<String> { Err(crate::FieldErr("id")) }
	fn base_type(&self) -> crate::Field<BaseType> { Err(crate::FieldErr("type")) }
}


pub trait BaseMut : crate::macros::MaybeSend {
	fn set_id(self, val: Option<String>) -> Self;
	fn set_base_type(self, val: Option<BaseType>) -> Self;
}


impl Base for String {
	fn id(&self) -> crate::Field<String> {
		Ok(self.clone())
	}

	fn base_type(&self) -> crate::Field<BaseType> {
		Ok(BaseType::Link(LinkType::Link))
	}
}

#[cfg(feature = "unstructured")]
impl Base for serde_json::Value {
	fn base_type(&self) -> crate::Field<BaseType> {
		if self.is_string() {
			Ok(BaseType::Link(LinkType::Link))
		} else {
			self.get("type")
				.and_then(|x| x.as_str())
				.and_then(|x| x.try_into().ok())
				.ok_or(crate::FieldErr("type"))
		}
	}

	fn id(&self) -> crate::Field<String> {
		if self.is_string() {
			Ok(self.as_str().ok_or(crate::FieldErr("id"))?.to_string())
		} else {
			self.get("id")
				.and_then(|x| x.as_str())
				.map(|x| x.to_string())
				.ok_or(crate::FieldErr("id"))
		}
	}
}

#[cfg(feature = "unstructured")]
impl BaseMut for serde_json::Value {
	crate::setter! { id -> String }
	crate::setter! { base_type -> type BaseType }
}
