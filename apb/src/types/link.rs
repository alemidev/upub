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
	fn href(&self) -> Field<String>;
	fn rel(&self) -> Field<String> { Err(FieldErr("rel")) }
	fn media_type(&self) -> Field<String> { Err(FieldErr("mediaType")) } // also in obj
	fn name(&self) -> Field<String> { Err(FieldErr("name")) }       // also in obj
	fn hreflang(&self) -> Field<String> { Err(FieldErr("hreflang")) }
	fn height(&self) -> Field<u64> { Err(FieldErr("height")) }
	fn width(&self) -> Field<u64> { Err(FieldErr("width")) }
	fn preview(&self) -> Field<String> { Err(FieldErr("linkPreview")) }    // also in obj
}

pub trait LinkMut : crate::BaseMut {
	fn set_link_type(self, val: Option<LinkType>) -> Self;
	fn set_href(self, href: Option<String>) -> Self;
	fn set_rel(self, val: Option<String>) -> Self;
	fn set_media_type(self, val: Option<String>) -> Self; // also in obj
	fn set_name(self, val: Option<String>) -> Self;       // also in obj
	fn set_hreflang(self, val: Option<String>) -> Self;
	fn set_height(self, val: Option<u64>) -> Self;
	fn set_width(self, val: Option<u64>) -> Self;
	fn set_preview(self, val: Option<String>) -> Self;    // also in obj
}

impl Link for String {
	fn href(&self) -> Field<String> {
		Ok(self.to_string())
	}
}

#[cfg(feature = "unstructured")]
impl Link for serde_json::Value {
	// TODO this can fail, but it should never do!
	fn href(&self) -> Field<String> {
		if self.is_string() {
			self.as_str().map(|x| x.to_string()).ok_or(FieldErr("href"))
		} else {
			self.get("href")
				.and_then(|x| x.as_str())
				.map(|x| x.to_string())
				.ok_or(FieldErr("href"))
		}
	}

	crate::getter! { link_type -> type LinkType }
	crate::getter! { rel -> String }
	crate::getter! { mediaType -> String }
	crate::getter! { name -> String }
	crate::getter! { hreflang -> String }
	crate::getter! { height -> u64 }
	crate::getter! { width -> u64 }
	crate::getter! { preview -> String }
}

#[cfg(feature = "unstructured")]
impl LinkMut for serde_json::Value {
	fn set_href(mut self, href: Option<String>) -> Self {
		match &mut self {
			serde_json::Value::Object(map) => {
				match href {
					Some(href) => map.insert(
						"href".to_string(),
						serde_json::Value::String(href)
					),
					None => map.remove("href"),
				};
			},
			x => *x = serde_json::Value::String(href.unwrap_or_default()),
		}
		self
	}

	crate::setter! { link_type -> type LinkType }
	crate::setter! { rel -> String }
	crate::setter! { mediaType -> String }
	crate::setter! { name -> String }
	crate::setter! { hreflang -> String }
	crate::setter! { height -> u64 }
	crate::setter! { width -> u64 }
	crate::setter! { preview -> String }
}
