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


pub trait Place : super::Object {
	fn accuracy(&self) -> Option<f32> { None }
	fn altitude(&self) -> Option<f32> { None }
	fn latitude(&self) -> Option<f32> { None }
	fn longitude(&self) -> Option<f32> { None }
	fn radius(&self) -> Option<f32> { None }
	fn units(&self) -> Option<&str> { None }
}

pub trait Image : Document {}

impl Document for serde_json::Value {

}

impl Image for serde_json::Value {}

