use crate::{getter, setter, strenum};

strenum! {
	pub enum LinkType {
		Link,
		Mention;
	};
}

pub trait Link : super::Base {
	fn href(&self) -> &str;
	fn rel(&self) -> Option<&str> { None }
	fn link_media_type(&self) -> Option<&str> { None } // also in obj
	fn link_name(&self) -> Option<&str> { None }       // also in obj
	fn hreflang(&self) -> Option<&str> { None }
	fn height(&self) -> Option<u64> { None }
	fn width(&self) -> Option<u64> { None }
	fn link_preview(&self) -> Option<&str> { None }    // also in obj
}

pub trait LinkMut : super::BaseMut {
	fn set_href(self, href: &str) -> Self;
	fn set_rel(self, val: Option<&str>) -> Self;
	fn set_link_media_type(self, val: Option<&str>) -> Self; // also in obj
	fn set_link_name(self, val: Option<&str>) -> Self;       // also in obj
	fn set_hreflang(self, val: Option<&str>) -> Self;
	fn set_height(self, val: Option<u64>) -> Self;
	fn set_width(self, val: Option<u64>) -> Self;
	fn set_link_preview(self, val: Option<&str>) -> Self;    // also in obj
}

impl Link for String {
	fn href(&self) -> &str {
		self
	}
}

impl Link for serde_json::Value {
	// TODO this can fail, but it should never do!
	fn href(&self) -> &str {
		match self {
			serde_json::Value::String(x) => x,
			serde_json::Value::Object(map) =>
				map.get("href")
					.map(|h| h.as_str().unwrap_or(""))
					.unwrap_or(""),
			_ => {
				tracing::error!("failed getting href on invalid json Link object");
				""
			},
		}
	}

	getter! { rel -> &str }
	getter! { link_media_type::mediaType -> &str }
	getter! { link_name::name -> &str }
	getter! { hreflang -> &str }
	getter! { height -> u64 }
	getter! { width -> u64 }
	getter! { link_preview::preview -> &str }
}

impl LinkMut for serde_json::Value {
	// TODO this can fail, but it should never do!
	fn set_href(mut self, href: &str) -> Self {
		match &mut self {
			serde_json::Value::String(x) => *x = href.to_string(),
			serde_json::Value::Object(map) => {
				map.insert(
					"href".to_string(),
					serde_json::Value::String(href.to_string())
				);
			},
			_ => tracing::error!("failed setting href on invalid json Link object"),
		}
		self
	}

	setter! { rel -> &str }
	setter! { link_media_type::mediaType -> &str }
	setter! { link_name::name -> &str }
	setter! { hreflang -> &str }
	setter! { height -> u64 }
	setter! { width -> u64 }
	setter! { link_preview::preview -> &str }
}
