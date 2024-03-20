pub trait Place : super::Object {
	fn accuracy(&self) -> Option<f64> { None }
	fn altitude(&self) -> Option<f64> { None }
	fn latitude(&self) -> Option<f64> { None }
	fn longitude(&self) -> Option<f64> { None }
	fn radius(&self) -> Option<f64> { None }
	fn units(&self) -> Option<&str> { None }
}

pub trait PlaceMut : super::ObjectMut {
	fn set_accuracy(&mut self, val: Option<f64>) -> &mut Self;
	fn set_altitude(&mut self, val: Option<f64>) -> &mut Self;
	fn set_latitude(&mut self, val: Option<f64>) -> &mut Self;
	fn set_longitude(&mut self, val: Option<f64>) -> &mut Self;
	fn set_radius(&mut self, val: Option<f64>) -> &mut Self;
	fn set_units(&mut self, val: Option<&str>) -> &mut Self;
}
