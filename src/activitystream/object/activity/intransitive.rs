use crate::strenum;

strenum! {
	pub enum IntransitiveActivityType {
		IntransitiveActivity,
		Arrive,
		Question,
		Travel
	}
}

pub trait IntransitiveActivity : super::Activity {
	fn intransitive_activity_type(&self) -> Option<IntransitiveActivityType>;
}
