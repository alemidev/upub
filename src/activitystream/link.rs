pub trait Link {
	fn href(&self) -> &str;
	fn rel(&self) -> Option<&str> { None }
	fn media_type(&self) -> Option<&str> { None } // also in obj
	fn name(&self) -> Option<&str> { None }       // also in obj
	fn hreflang(&self) -> Option<&str> { None }
	fn height(&self) -> Option<&str> { None }
	fn width(&self) -> Option<&str> { None }
	fn preview(&self) -> Option<&str> { None }    // also in obj
}

pub enum LinkedObject<T> {
	Object(T),
	Link(Box<dyn Link>),
}

impl<T> LinkedObject<T>
where
	T : for<'de> serde::Deserialize<'de>,
{
	pub async fn resolve(self) -> T {
		match self {
			LinkedObject::Object(o) => o,
			LinkedObject::Link(l) =>
				reqwest::get(l.href())
					.await.unwrap()
					.json::<T>()
					.await.unwrap(),
		}
	}
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

impl From<serde_json::Value> for LinkedObject<serde_json::Value> {
	fn from(value: serde_json::Value) -> Self {
		if value.is_string() || value.get("href").is_some() {
			Self::Link(Box::new(value))
		} else {
			Self::Object(value)
		}
	}
}

