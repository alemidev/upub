use crate::{Field, FieldErr};

pub trait Place : super::Object {
	fn accuracy(&self) -> Field<f64> { Err(FieldErr("accuracy")) }
	fn altitude(&self) -> Field<f64> { Err(FieldErr("altitude")) }
	fn latitude(&self) -> Field<f64> { Err(FieldErr("latitude")) }
	fn longitude(&self) -> Field<f64> { Err(FieldErr("longitude")) }
	fn radius(&self) -> Field<f64> { Err(FieldErr("radius")) }
	fn units(&self) -> Field<&str> { Err(FieldErr("units")) }
}

pub trait PlaceMut : super::ObjectMut {
	fn set_accuracy(self, val: Option<f64>) -> Self;
	fn set_altitude(self, val: Option<f64>) -> Self;
	fn set_latitude(self, val: Option<f64>) -> Self;
	fn set_longitude(self, val: Option<f64>) -> Self;
	fn set_radius(self, val: Option<f64>) -> Self;
	fn set_units(self, val: Option<&str>) -> Self;
}
