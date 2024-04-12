use crate::strenum;

strenum! {
	pub enum AcceptType {
		Accept,
		TentativeAccept;
	};
}

pub trait Accept : super::Activity {
	fn accept_type(&self) -> Option<AcceptType> { None }
}

pub trait AcceptMut : super::ActivityMut {
	fn set_accept_type(self, val: Option<AcceptType>) -> Self;
}