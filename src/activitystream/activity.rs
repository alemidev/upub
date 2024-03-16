pub trait Activity : super::Object {
	fn activity_type(&self) -> Option<super::types::ActivityType> { None }

	fn actor_id(&self) -> Option<&str> { None }
	fn actor(&self) -> Option<&impl super::Object> { None::<&()> }

	fn object_id(&self) -> Option<&str> { None }
	fn object(&self) -> Option<&impl super::Object> { None::<&()> }

	fn target(&self) -> Option<&str> { None }
}
