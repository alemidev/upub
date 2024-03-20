use crate::strenum;

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
	fn set_document_type(&mut self, val: Option<DocumentType>) -> &mut Self;
}

pub trait Image : Document {}



impl Document for serde_json::Value {}
impl Image for serde_json::Value {}

