use crate::strenum;

strenum! {
	pub enum LinkType {
		Link,
		Mention
	}
}

pub trait Link : super::Base {
	fn href(&self) -> &str;
	fn rel(&self) -> Option<&str> { None }
	fn media_type(&self) -> Option<&str> { None } // also in obj
	fn name(&self) -> Option<&str> { None }       // also in obj
	fn hreflang(&self) -> Option<&str> { None }
	fn height(&self) -> Option<&str> { None }
	fn width(&self) -> Option<&str> { None }
	fn preview(&self) -> Option<&str> { None }    // also in obj
}

impl Link for String {
	fn href(&self) -> &str {
		self
	}
}

impl Link for serde_json::Value {
	// TODO this is unchecked and can panic
	fn href(&self) -> &str {
		match self {
			serde_json::Value::String(x) => x,
			serde_json::Value::Object(map) =>
				map.get("href")
					.unwrap()
					.as_str()
					.unwrap(),
			_ => panic!("invalid value for Link"),

		}
	}

	// ... TODO!
}
