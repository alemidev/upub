use crate::strenum;

strenum! {
	pub enum IgnoreType {
		Ignore,
		Block;
	};
}

pub trait Ignore : super::Activity {
	fn ignore_type(&self) -> Option<IgnoreType> { None }
}

pub trait IgnoreMut : super::ActivityMut {
	fn set_ignore_type(self, val: Option<IgnoreType>) -> Self;
}
