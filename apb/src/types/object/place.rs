pub trait Place : super::Object {
	fn accuracy(&self) -> Option<f64> { None }
	fn altitude(&self) -> Option<f64> { None }
	fn latitude(&self) -> Option<f64> { None }
	fn longitude(&self) -> Option<f64> { None }
	fn radius(&self) -> Option<f64> { None }
	fn units(&self) -> Option<&str> { None }
}

pub trait PlaceMut : super::ObjectMut {
	fn set_accuracy(self, val: Option<f64>) -> Self;
	fn set_altitude(self, val: Option<f64>) -> Self;
	fn set_latitude(self, val: Option<f64>) -> Self;
	fn set_longitude(self, val: Option<f64>) -> Self;
	fn set_radius(self, val: Option<f64>) -> Self;
	fn set_units(self, val: Option<&str>) -> Self;
}
