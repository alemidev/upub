pub mod types;

pub mod link;
pub use link::{Link, LinkType};

pub mod object;
pub use object::{Object, ObjectType};

pub mod node;
pub use node::Node;

use crate::strenum;

strenum! {
	pub enum BaseType {
		Invalid

		Object(ObjectType),
		Link(LinkType)
	}
}

pub trait Base {
	fn id(&self) -> Option<&str> { None }
	fn base_type(&self) -> Option<BaseType> { None }
}

impl Base for () {}

impl Base for String {
	fn id(&self) -> Option<&str> {
		Some(self)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Link(LinkType::Link))
	}
}

impl Base for serde_json::Value {
	fn id(&self) -> Option<&str> {
		self.get("id")?.as_str()
	}

	fn base_type(&self) -> Option<BaseType> {
		self.get("type")?.as_str()?.try_into().ok()
	}
}
