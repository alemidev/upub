pub trait Activity : super::Object {
	fn actor(&self) -> Option<&super::ObjectOrLink> { None }
	fn object(&self) -> Option<&super::ObjectOrLink> { None }
}
