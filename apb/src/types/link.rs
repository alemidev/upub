use crate::{Field, FieldErr};

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
	fn link_type(&self) -> Field<LinkType> { Err(FieldErr("type")) }
	fn href(&self) -> &str;
	fn rel(&self) -> Field<&str> { Err(FieldErr("rel")) }
	fn media_type(&self) -> Field<&str> { Err(FieldErr("mediaType")) } // also in obj
	fn name(&self) -> Field<&str> { Err(FieldErr("name")) }       // also in obj
	fn hreflang(&self) -> Field<&str> { Err(FieldErr("hreflang")) }
	fn height(&self) -> Field<u64> { Err(FieldErr("height")) }
	fn width(&self) -> Field<u64> { Err(FieldErr("width")) }
	fn preview(&self) -> Field<&str> { Err(FieldErr("linkPreview")) }    // also in obj
}

pub trait LinkMut : crate::BaseMut {
	fn set_link_type(self, val: Option<LinkType>) -> Self;
	fn set_href(self, href: &str) -> Self;
	fn set_rel(self, val: Option<&str>) -> Self;
	fn set_media_type(self, val: Option<&str>) -> Self; // also in obj
	fn set_name(self, val: Option<&str>) -> Self;       // also in obj
	fn set_hreflang(self, val: Option<&str>) -> Self;
	fn set_height(self, val: Option<u64>) -> Self;
	fn set_width(self, val: Option<u64>) -> Self;
	fn set_preview(self, val: Option<&str>) -> Self;    // also in obj
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

	crate::getter! { link_type -> type LinkType }
	crate::getter! { rel -> &str }
	crate::getter! { mediaType -> &str }
	crate::getter! { name -> &str }
	crate::getter! { hreflang -> &str }
	crate::getter! { height -> u64 }
	crate::getter! { width -> u64 }
	crate::getter! { preview -> &str }
}

#[cfg(feature = "unstructured")]
impl LinkMut for serde_json::Value {
	fn set_href(mut self, href: &str) -> Self {
		match &mut self {
			serde_json::Value::Object(map) => {
				map.insert(
					"href".to_string(),
					serde_json::Value::String(href.to_string())
				);
			},
			x => *x = serde_json::Value::String(href.to_string()),
		}
		self
	}

	crate::setter! { link_type -> type LinkType }
	crate::setter! { rel -> &str }
	crate::setter! { mediaType -> &str }
	crate::setter! { name -> &str }
	crate::setter! { hreflang -> &str }
	crate::setter! { height -> u64 }
	crate::setter! { width -> u64 }
	crate::setter! { preview -> &str }
}
