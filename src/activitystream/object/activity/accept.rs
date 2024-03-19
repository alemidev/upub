use crate::strenum;

strenum! {
	pub enum AcceptType {
		Accept,
		TentativeAccept;
	};
}

pub trait Accept : super::Activity {
	fn accept_type(&self) -> Option<AcceptType>;
}
