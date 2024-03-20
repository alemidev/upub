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
	fn intransitive_activity_type(&self) -> Option<IntransitiveActivityType> { None }
}

pub trait IntransitiveActivityMut : super::ActivityMut {
	fn set_intransitive_activity_type(&mut self, val: Option<IntransitiveActivityType>) -> &mut Self;
}
