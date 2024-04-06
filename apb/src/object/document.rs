use crate::{getter, setter, strenum};

strenum! {
	pub enum DocumentType {
		Document,
		Audio,
		Image,
		Page,
		Video;
	};
}

pub trait Document : super::Object {
	fn document_type(&self) -> Option<DocumentType> { None }
}

pub trait DocumentMut : super::ObjectMut {
	fn set_document_type(self, val: Option<DocumentType>) -> Self;
}


impl Document for serde_json::Value {
	getter! { document_type -> type DocumentType }
}

impl DocumentMut for serde_json::Value {
	setter! { document_type -> type DocumentType }
}
