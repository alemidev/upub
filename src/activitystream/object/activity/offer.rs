use crate::strenum;

strenum! {
	pub enum OfferType {
		Offer,
		Invite;
	};
}

pub trait Offer : super::Activity {
	fn offer_type(&self) -> Option<OfferType> { None }
}

pub trait OfferMut : super::ActivityMut {
	fn set_offer_type(&mut self, val: Option<OfferType>) -> &mut Self;
}
