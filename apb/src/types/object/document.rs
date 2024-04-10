crate::strenum! {
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


#[cfg(feature = "unstructured")]
impl Document for serde_json::Value {
	crate::getter! { document_type -> type DocumentType }
}

#[cfg(feature = "unstructured")]
impl DocumentMut for serde_json::Value {
	crate::setter! { document_type -> type DocumentType }
}
