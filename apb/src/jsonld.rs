use crate::Object;

pub trait LD {
	fn ld_context(self) -> Self;
}

impl LD for serde_json::Value {
	fn ld_context(mut self) -> Self {
		let o_type = self.object_type();
		if let Some(obj) = self.as_object_mut() {
			let mut ctx = serde_json::Map::new();
			ctx.insert("sensitive".to_string(), serde_json::Value::String("as:sensitive".into()));
			ctx.insert("quoteUrl".to_string(), serde_json::Value::String("as:quoteUrl".into()));
			match o_type {
				Ok(crate::ObjectType::Actor(_)) => {
					ctx.insert("counters".to_string(), serde_json::Value::String("https://ns.alemi.dev/as/counters/#".into()));
					ctx.insert("followingCount".to_string(), serde_json::Value::String("counters:followingCount".into()));
					ctx.insert("followersCount".to_string(), serde_json::Value::String("counters:followersCount".into()));
					ctx.insert("statusesCount".to_string(), serde_json::Value::String("counters:statusesCount".into()));
					ctx.insert("fe".to_string(), serde_json::Value::String("https://ns.alemi.dev/as/fe/#".into()));
					ctx.insert("followingMe".to_string(), serde_json::Value::String("fe:followingMe".into()));
					ctx.insert("followedByMe".to_string(), serde_json::Value::String("fe:followedByMe".into()));
				},
				Ok(
					crate::ObjectType::Note
					| crate::ObjectType::Article
					| crate::ObjectType::Event
					| crate::ObjectType::Document(crate::DocumentType::Page) // TODO why Document lemmyyyyyy
				) => {
					ctx.insert("fe".to_string(), serde_json::Value::String("https://ns.alemi.dev/as/fe/#".into()));
					ctx.insert("likedByMe".to_string(), serde_json::Value::String("fe:likedByMe".into()));
					ctx.insert("ostatus".to_string(), serde_json::Value::String("http://ostatus.org#".into()));
					ctx.insert("conversation".to_string(), serde_json::Value::String("ostatus:conversation".into()));
				},
				_ => {},
			}
			obj.insert(
				"@context".to_string(),
				serde_json::Value::Array(vec![
					serde_json::Value::String("https://www.w3.org/ns/activitystreams".into()),
					serde_json::Value::String("https://w3id.org/security/v1".into()),
					serde_json::Value::Object(ctx),
				]),
			);
		} else {
			tracing::warn!("cannot add @context to json value different than object");
		}
		self
	}
}
