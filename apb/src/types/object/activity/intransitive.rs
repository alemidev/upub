use crate::strenum;

strenum! {
	pub enum IntransitiveActivityType {
		IntransitiveActivity,
		Arrive,
		Question,
		Travel;
	};
}

pub trait IntransitiveActivity : super::Activity {
	fn intransitive_activity_type(&self) -> crate::Field<IntransitiveActivityType> { Err(crate::FieldErr("type")) }
}

pub trait IntransitiveActivityMut : super::ActivityMut {
	fn set_intransitive_activity_type(self, val: Option<IntransitiveActivityType>) -> Self;
}
