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
	fn intransitive_activity_type(&self) -> crate::Field<IntransitiveActivityType> { Err(crate::FieldErr("type", None)) }
}

pub trait IntransitiveActivityMut : super::ActivityMut {
	fn set_intransitive_activity_type(self, val: Option<IntransitiveActivityType>) -> Self;
}

pub trait Question : super::Activity {
	fn any_of(&self) -> crate::Node<Self::Object> { crate::Node::Empty }
	fn one_of(&self) -> crate::Node<Self::Object> { crate::Node::Empty }
}

pub trait QuestionMut : super::ActivityMut {
	fn set_any_of(self, val: crate::Node<Self::Object>) -> Self;
	fn set_one_of(self, val: crate::Node<Self::Object>) -> Self;
}

#[cfg(feature = "unstructured")]
impl Question for serde_json::Value {
	crate::getter! { anyOf -> node <Self as super::Object>::Object }
	crate::getter! { oneOf -> node <Self as super::Object>::Object }
}

#[cfg(feature = "unstructured")]
impl QuestionMut for serde_json::Value {
	crate::setter! { anyOf -> node <Self as super::Object>::Object }
	crate::setter! { oneOf -> node <Self as super::Object>::Object }
}
