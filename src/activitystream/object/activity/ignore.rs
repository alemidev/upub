use crate::strenum;

strenum! {
	pub enum IgnoreType {
		Ignore,
		Block;
	};
}

pub trait Ignore : super::Activity {
	fn ignore_type(&self) -> Option<IgnoreType>;
}
