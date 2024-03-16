pub enum ObjectOrLink {
	Object(Box<dyn Object>),
	Link(Box<dyn Link>),
}

impl From<serde_json::Value> for ObjectOrLink {
	fn from(value: serde_json::Value) -> Self {
		if value.get("href").is_some() {
			Self::Link(Box::new(value))
		} else {
			Self::Object(Box::new(value))
		}
	}
}

pub trait Link {
	fn href(&self) -> Option<&str> { None }
	fn rel(&self) -> Option<&str> { None }
	fn media_type(&self) -> Option<&str> { None } // also in obj
	fn name(&self) -> Option<&str> { None }       // also in obj
	fn hreflang(&self) -> Option<&str> { None }
	fn height(&self) -> Option<&str> { None }
	fn width(&self) -> Option<&str> { None }
	fn preview(&self) -> Option<&str> { None }    // also in obj
}


pub trait Object {
	fn id(&self) -> Option<&str> { None }
	fn full_type(&self) -> Option<super::Type> { None }
	fn attachment (&self) -> Option<&str> { None }
	fn attributed_to (&self) -> Option<&str> { None }
	fn audience (&self) -> Option<&str> { None }
	fn content (&self) -> Option<&str> { None } 
	fn context (&self) -> Option<&str> { None }
	fn name (&self) -> Option<&str> { None }
	fn end_time (&self) -> Option<&str> { None }
	fn generator (&self) -> Option<&str> { None }
	fn icon (&self) -> Option<&str> { None }
	fn image (&self) -> Option<&str> { None }
	fn in_reply_to (&self) -> Option<&str> { None }
	fn location (&self) -> Option<&str> { None }
	fn preview (&self) -> Option<&str> { None }
	fn published (&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
	fn replies (&self) -> Option<&str> { None }
	fn start_time (&self) -> Option<&str> { None }
	fn summary (&self) -> Option<&str> { None }
	fn tag (&self) -> Option<&str> { None }
	fn updated (&self) -> Option<&str> { None }
	fn url (&self) -> Option<&str> { None }
	fn to (&self) -> Option<&str> { None }
	fn bto (&self) -> Option<&str> { None }
	fn cc (&self) -> Option<&str> { None }
	fn bcc (&self) -> Option<&str> { None }
	fn media_type (&self) -> Option<&str> { None }
	fn duration (&self) -> Option<&str> { None }
}

/// impl for empty object
impl Object for () {}

// TODO only Value::Object is a valid Object, but rn "asd" behaves like {} (both are valid...)
/// impl for any json value
impl Object for serde_json::Value {
	fn id(&self) -> Option<&str> {
		self.get("id")?.as_str()
	}

	fn full_type(&self) -> Option<super::Type> {
		todo!()
	}

	// ... TODO!
}

impl Link for serde_json::Value {
	fn href(&self) -> Option<&str> {
		self.get("href")?.as_str()
	}

	// ... TODO!
}

pub trait ToJson : Object {
	fn json(&self) -> serde_json::Value;
}

impl<T> ToJson for T where T : Object {
	fn json(&self) -> serde_json::Value {
		let mut map = serde_json::Map::new();
		let mp = &mut map;

		put_str(mp, "id", self.id());
		put_str(mp, "attributedTo", self.attributed_to());
		put_str(mp, "name", self.name());
		put_str(mp, "summary", self.summary());
		put_str(mp, "content", self.content());

		if let Some(t) = self.full_type() {
			map.insert(
				"type".to_string(),
				serde_json::Value::String(format!("{t}")),
			);
		}

		if let Some(published) = self.published() {
			map.insert(
				"published".to_string(),
				serde_json::Value::String(published.to_rfc3339()),
			);
		}

		// ... TODO!

		serde_json::Value::Object(map)
	}
}

fn put_str(map: &mut serde_json::Map<String, serde_json::Value>, k: &str, v: Option<&str>) {
	if let Some(v) = v {
		map.insert(
			k.to_string(),
			serde_json::Value::String(v.to_string()),
		);
	}
}
