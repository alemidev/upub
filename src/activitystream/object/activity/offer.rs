use crate::strenum;

strenum! {
	pub enum OfferType {
		Offer,
		Invite
	}
}

pub trait Offer : super::Activity {
	fn offer_type(&self) -> Option<OfferType>;
}
