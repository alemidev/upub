pub trait Actor : super::Object {
	fn actor_type(&self) -> Option<super::ActorType> { None }
}
