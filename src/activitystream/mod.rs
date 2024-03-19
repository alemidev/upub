pub mod object;
pub use object::Object;


pub mod actor;
pub use actor::Actor;


pub mod activity;
pub use activity::Activity;


pub mod types;
pub use types::{BaseType, ObjectType, ActivityType, ActorType};


pub mod link;
pub use link::{Link, LinkedObject};

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
				serde_json::Value::String(format!("{t:?}")),
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
