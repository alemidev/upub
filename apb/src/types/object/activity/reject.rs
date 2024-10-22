use crate::strenum;

strenum! {
	pub enum RejectType {
		Reject,
		TentativeReject;
	};
}

pub trait Reject : super::Activity {
	fn reject_type(&self) -> crate::Field<RejectType> { Err(crate::FieldErr("type")) }
}

pub trait RejectMut : super::ActivityMut {
	fn set_reject_type(self, val: Option<RejectType>) -> Self;
}
