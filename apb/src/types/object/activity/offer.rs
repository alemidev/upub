use crate::strenum;

strenum! {
	pub enum OfferType {
		Offer,
		Invite;
	};
}

pub trait Offer : super::Activity {
	fn offer_type(&self) -> crate::Field<OfferType> { Err(crate::FieldErr("type")) }
}

pub trait OfferMut : super::ActivityMut {
	fn set_offer_type(self, val: Option<OfferType>) -> Self;
}
