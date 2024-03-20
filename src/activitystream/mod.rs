pub mod link;
pub use link::{Link, LinkType};

pub mod object;
pub use object::{Object, ObjectType};

pub mod node;
pub use node::Node;

pub mod macros;

use crate::{getter, setter, strenum};

strenum! {
	pub enum BaseType {
		;
		Object(ObjectType),
		Link(LinkType)
	};
}

pub trait Base {
	fn id(&self) -> Option<&str> { None }
	fn base_type(&self) -> Option<BaseType> { None }

	// TODO this is a dirty fix because my trait model is flawed and leads to circular resolution
	//      errors, basically can't downcast back to serde_json::Value once i've updasted it to
	//      impl Object/Actor/whatever... ask me to infodump+bikeshed about this!!! :3
	fn underlying_json_object(self) -> serde_json::Value;
}

pub fn object() -> serde_json::Value {
	let mut map = serde_json::Map::default();
	map.insert(
		"@context".to_string(),
		serde_json::Value::Array(vec![
			serde_json::Value::String("https://www.w3.org/ns/activitystreams".into())
		]),
	);
	serde_json::Value::Object(map)
}

pub trait BaseMut {
	fn set_id(&mut self, val: Option<&str>) -> &mut Self;
	fn set_base_type(&mut self, val: Option<BaseType>) -> &mut Self;
}


impl Base for String {
	fn id(&self) -> Option<&str> {
		Some(self)
	}

	fn base_type(&self) -> Option<BaseType> {
		Some(BaseType::Link(LinkType::Link))
	}

	fn underlying_json_object(self) -> serde_json::Value {
		serde_json::Value::String(self)
	}
}

impl Base for serde_json::Value {
	fn underlying_json_object(self) -> serde_json::Value {
		self
	}

	getter! { id -> &str }
	getter! { base_type -> type BaseType }
}

impl BaseMut for serde_json::Value {
	setter! { id -> &str }
	setter! { base_type -> type BaseType }
}
