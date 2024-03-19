use crate::strenum;

strenum! {
	pub enum RejectType {
		Reject,
		TentativeReject;
	};
}

pub trait Reject : super::Activity {
	fn reject_type(&self) -> Option<RejectType>;
}
