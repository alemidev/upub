
pub trait Getter<T: Default> {
	/// .ok().unwrap_or_default()
	fn want(self) -> T;
}

impl<T: Default> Getter<T> for apb::Field<T> {
	fn want(self) -> T {
		self.ok().unwrap_or_default()
	}
}
