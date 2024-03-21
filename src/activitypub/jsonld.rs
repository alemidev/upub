use axum::response::{IntoResponse, Response};

pub trait LD {
	fn ld_context(self) -> Self;
	fn new_object() -> serde_json::Value {
		serde_json::Value::Object(serde_json::Map::default())
	}
}

impl LD for serde_json::Value {
	fn ld_context(mut self) -> Self {
		if let Some(obj) = self.as_object_mut() {
			obj.insert(
				"@context".to_string(),
				serde_json::Value::Array(vec![
					serde_json::Value::String("https://www.w3.org/ns/activitystreams".into())
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
		// headers                                  body
		([("Content-Type", "application/ld+json")], axum::Json(self.0)).into_response()
	}
}
