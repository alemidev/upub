pub trait Activity : super::Object {
	fn activity_type(&self) -> Option<super::types::ActivityType> { None }

	fn actor_id(&self) -> Option<&str> { None }
	fn actor(&self) -> Option<super::LinkedObject<impl super::Object>> { None::<super::LinkedObject<()>> }

	fn object_id(&self) -> Option<&str> { None }
	fn object(&self) -> Option<super::LinkedObject<impl super::Object>> { None::<super::LinkedObject<()>> }

	fn target(&self) -> Option<&str> { None }
}

impl Activity for serde_json::Value {
	fn activity_type(&self) -> Option<super::types::ActivityType> {
		let serde_json::Value::String(t) = self.get("type")? else { return None };
		super::types::ActivityType::try_from(t.as_str()).ok()
	}

	fn object(&self) -> Option<super::LinkedObject<impl super::Object>> {
		let obj = self.get("object")?;
		match obj {
			serde_json::Value::Object(_) => Some(obj.clone().into()),
			_ => None,
		}
	}

	fn object_id(&self) -> Option<&str> {
		match self.get("object")? {
			serde_json::Value::Object(map) => match map.get("id")? {
				serde_json::Value::String(id) => Some(id),
				_ => None,
			},
			serde_json::Value::String(id) => Some(id),
			_ => None,
		}
	}

	fn actor(&self) -> Option<super::LinkedObject<impl super::Object>> {
		let obj = self.get("actor")?;
		match obj {
			serde_json::Value::Object(_) => Some(obj.clone().into()),
			_ => None,
		}
	}

	fn actor_id(&self) -> Option<&str> {
		match self.get("actor")? {
			serde_json::Value::Object(map) => match map.get("id")? {
				serde_json::Value::String(id) => Some(id),
				_ => None,
			},
			serde_json::Value::String(id) => Some(id),
			_ => None,
		}
	}
}
