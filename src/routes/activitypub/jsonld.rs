// TODO
// move this file somewhere else
// it's not a route
// maybe under src/server/jsonld.rs ??

use apb::Object;
use axum::response::{IntoResponse, Response};

pub trait LD {
	fn ld_context(self) -> Self;
	fn new_object() -> serde_json::Value {
		serde_json::Value::Object(serde_json::Map::default())
	}
}

impl LD for serde_json::Value {
	fn ld_context(mut self) -> Self {
		let o_type = self.object_type();
		if let Some(obj) = self.as_object_mut() {
			let mut ctx = serde_json::Map::new();
			ctx.insert("sensitive".to_string(), serde_json::Value::String("as:sensitive".into()));
			ctx.insert("quoteUrl".to_string(), serde_json::Value::String("as:quoteUrl".into()));
			match o_type {
				Some(apb::ObjectType::Actor(_)) => {
					ctx.insert("counters".to_string(), serde_json::Value::String("https://ns.alemi.dev/as/counters/#".into()));
					ctx.insert("followingCount".to_string(), serde_json::Value::String("counters:followingCount".into()));
					ctx.insert("followersCount".to_string(), serde_json::Value::String("counters:followersCount".into()));
					ctx.insert("statusesCount".to_string(), serde_json::Value::String("counters:statusesCount".into()));
					ctx.insert("fe".to_string(), serde_json::Value::String("https://ns.alemi.dev/as/fe/#".into()));
					ctx.insert("followingMe".to_string(), serde_json::Value::String("fe:followingMe".into()));
					ctx.insert("followedByMe".to_string(), serde_json::Value::String("fe:followedByMe".into()));
				},
				Some(_) => {
					ctx.insert("fe".to_string(), serde_json::Value::String("https://ns.alemi.dev/as/fe/#".into()));
					ctx.insert("likedByMe".to_string(), serde_json::Value::String("fe:likedByMe".into()));
				},
				None => {},
			}
			obj.insert(
				"@context".to_string(),
				serde_json::Value::Array(vec![
					serde_json::Value::String("https://www.w3.org/ns/activitystreams".into()),
					serde_json::Value::Object(ctx),
				]),
			);
		} else {
			tracing::warn!("cannot add @context to json value different than object");
		}
		self
	}
}

// got this from https://github.com/kitsune-soc/kitsune/blob/b023a12b687dd9a274233a5a9950f2de5e192344/kitsune/src/http/responder.rs
// i was trying to do it with middlewares but this is way cleaner
pub struct JsonLD<T>(pub T);
impl<T: serde::Serialize> IntoResponse for JsonLD<T> {
	fn into_response(self) -> Response {
		(
			[("Content-Type", "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")],
			axum::Json(self.0)
		).into_response()
	}
}
