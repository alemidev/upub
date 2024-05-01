#[cfg(feature = "activitypub-miscellaneous-terms")]
crate::strenum! {
	pub enum LinkType {
		Link,
		Hashtag,
		Mention;
	};
}

#[cfg(not(feature = "activitypub-miscellaneous-terms"))]
crate::strenum! {
	pub enum LinkType {
		Link,
		Mention;
	};
}

pub trait Link : crate::Base {
	fn href(&self) -> &str;
	fn rel(&self) -> Option<&str> { None }
	fn link_media_type(&self) -> Option<&str> { None } // also in obj
	fn link_name(&self) -> Option<&str> { None }       // also in obj
	fn hreflang(&self) -> Option<&str> { None }
	fn height(&self) -> Option<u64> { None }
	fn width(&self) -> Option<u64> { None }
	fn link_preview(&self) -> Option<&str> { None }    // also in obj
}

pub trait LinkMut : crate::BaseMut {
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

#[cfg(feature = "unstructured")]
impl Link for serde_json::Value {
	// TODO this can fail, but it should never do!
	fn href(&self) -> &str {
		if self.is_string() {
			self.as_str().unwrap_or("")
		} else {
			self.get("href").map(|x| x.as_str().unwrap_or("")).unwrap_or("")
		}
	}

	crate::getter! { rel -> &str }
	crate::getter! { link_media_type::mediaType -> &str }
	crate::getter! { link_name::name -> &str }
	crate::getter! { hreflang -> &str }
	crate::getter! { height -> u64 }
	crate::getter! { width -> u64 }
	crate::getter! { link_preview::preview -> &str }
}

#[cfg(feature = "unstructured")]
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

	crate::setter! { rel -> &str }
	crate::setter! { link_media_type::mediaType -> &str }
	crate::setter! { link_name::name -> &str }
	crate::setter! { hreflang -> &str }
	crate::setter! { height -> u64 }
	crate::setter! { width -> u64 }
	crate::setter! { link_preview::preview -> &str }
}
