use crate::{Field, FieldErr};

pub trait Place : super::Object {
	fn accuracy(&self) -> Field<f64> { Err(FieldErr("accuracy", None)) }
	fn altitude(&self) -> Field<f64> { Err(FieldErr("altitude", None)) }
	fn latitude(&self) -> Field<f64> { Err(FieldErr("latitude", None)) }
	fn longitude(&self) -> Field<f64> { Err(FieldErr("longitude", None)) }
	fn radius(&self) -> Field<f64> { Err(FieldErr("radius", None)) }
	fn units(&self) -> Field<&str> { Err(FieldErr("units", None)) }
}

pub trait PlaceMut : super::ObjectMut {
	fn set_accuracy(self, val: Option<f64>) -> Self;
	fn set_altitude(self, val: Option<f64>) -> Self;
	fn set_latitude(self, val: Option<f64>) -> Self;
	fn set_longitude(self, val: Option<f64>) -> Self;
	fn set_radius(self, val: Option<f64>) -> Self;
	fn set_units(self, val: Option<&str>) -> Self;
}
