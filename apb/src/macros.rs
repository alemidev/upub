#[cfg(feature = "send")]
pub trait MaybeSend : Send {}
#[cfg(feature = "send")]
impl<T : Send> MaybeSend for T {}


#[cfg(not(feature = "send"))]
pub trait MaybeSend {}
#[cfg(not(feature = "send"))]
impl<T> MaybeSend for T {}


#[derive(Debug, thiserror::Error)]
#[error("invalid type value")]
pub struct TypeValueError;

#[cfg(feature = "orm")]
impl From<TypeValueError> for sea_orm::sea_query::ValueTypeErr {
	fn from(_: TypeValueError) -> Self {
		sea_orm::sea_query::ValueTypeErr
	}
}

#[cfg(feature = "orm")]
impl From<TypeValueError> for sea_orm::TryGetError {
	fn from(_: TypeValueError) -> Self {
		sea_orm::TryGetError::Null("value is not a valid type".into())
	}
}


macro_rules! strenum {
	( $(pub enum $enum_name:ident { $($flat:ident),* ; $($deep:ident($inner:ident)),* };)+ ) => {
		$(
			#[derive(PartialEq, Eq, Debug, Clone, Copy)]
			pub enum $enum_name {
				$($flat,)*
				$($deep($inner),)*
			}

			impl AsRef<str> for $enum_name {
				fn as_ref(&self) -> &str {
					match self {
						$(Self::$flat => stringify!($flat),)*
						$(Self::$deep(x) => x.as_ref(),)*
					}
				}
			}

			impl TryFrom<&str> for $enum_name {
				type Error = $crate::macros::TypeValueError;

				fn try_from(value:&str) -> Result<Self, Self::Error> {
					match value {
						$(stringify!($flat) => Ok(Self::$flat),)*
						_ => {
							$(
								if let Ok(x) = $inner::try_from(value) {
									return Ok(Self::$deep(x));
								}
							)*
							Err($crate::macros::TypeValueError)
						},
					}
				}
			}

			#[cfg(feature = "orm")]
			impl From<$enum_name> for sea_orm::Value {
				fn from(value: $enum_name) -> sea_orm::Value {
					sea_orm::Value::String(Some(Box::new(value.as_ref().to_string())))
				}
			}

			#[cfg(feature = "orm")]
			impl sea_orm::sea_query::ValueType for $enum_name {
				fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
					match v {
						sea_orm::Value::String(Some(x)) =>
							Ok(<Self as TryFrom<&str>>::try_from(x.as_str())?),
						_ => Err(sea_orm::sea_query::ValueTypeErr),
					}
				}
			
				fn type_name() -> String {
					stringify!($enum_name).to_string()
				}
			
				fn array_type() -> sea_orm::sea_query::ArrayType {
					sea_orm::sea_query::ArrayType::String
				}
			
				fn column_type() -> sea_orm::sea_query::ColumnType {
					sea_orm::sea_query::ColumnType::String(Some(24))
				}
			}
			
			#[cfg(feature = "orm")]
			impl sea_orm::TryGetable for $enum_name {
				fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::prelude::QueryResult, index: I) -> Result<Self, sea_orm::TryGetError> {
					let x : String = res.try_get_by(index)?;
					Ok(Self::try_from(x.as_str())?)
				}
			}
		)*
	};
}

pub(crate) use strenum;

macro_rules! getter {
	($name:ident -> type $t:ty) => {
		paste::paste! {
			fn [< $name:snake >] (&self) -> $crate::Field<$t> {
				self.get("type")
					.and_then(|x| x.as_str())
					.and_then(|x| x.try_into().ok())
					.ok_or($crate::FieldErr("type"))
			}
		}
	};

	($name:ident -> bool) => {
		paste::paste! {
			fn [< $name:snake >](&self) -> $crate::Field<bool> {
				self.get(stringify!($name))
					.and_then(|x| x.as_bool())
					.ok_or($crate::FieldErr(stringify!($name)))
			}
		}
	};

	($name:ident -> &str) => {
		paste::paste! {
			fn [< $name:snake >](&self) -> $crate::Field<&str> {
				self.get(stringify!($name))
					.and_then(|x| x.as_str())
					.ok_or($crate::FieldErr(stringify!($name)))
			}
		}
	};

	($name:ident -> f64) => {
		paste::paste! {
			fn [< $name:snake >](&self) -> $crate::Field<f64> {
				self.get(stringify!($name))
					.and_then(|x| x.as_f64())
					.ok_or($crate::FieldErr(stringify!($name)))
			}
		}
	};

	($name:ident -> u64) => {
		paste::paste! {
			fn [< $name:snake >](&self) -> $crate::Field<u64> {
				self.get(stringify!($name))
					.and_then(|x| x.as_u64())
					.ok_or($crate::FieldErr(stringify!($name)))
			}
		}
	};

	($name:ident -> i64) => {
		paste::paste! {
			fn [< $name:snake >](&self) -> $crate::Field<i64> {
				self.get(stringify!($name))
					.and_then(|x| x.as_i64())
					.ok_or($crate::FieldErr(stringify!($name)))
			}
		}
	};

	($name:ident -> chrono::DateTime<chrono::Utc>) => {
		paste::paste! {
			fn [< $name:snake >](&self) -> $crate::Field<chrono::DateTime<chrono::Utc>> {
				Ok(
					chrono::DateTime::parse_from_rfc3339(
							self
								.get(stringify!($name))
								.and_then(|x| x.as_str())
								.ok_or($crate::FieldErr(stringify!($name)))?
						)
						.map_err(|e| {
							tracing::warn!("invalid time string ({e}), ignoring");
							$crate::FieldErr(stringify!($name))
						})?
						.with_timezone(&chrono::Utc)
				)
			}
		}
	};

	($name:ident -> node $t:ty) => {
		paste::paste! {
			fn [< $name:snake >](&self) -> $crate::Node<$t> {
				match self.get(stringify!($name)) {
					Some(x) => $crate::Node::from(x.clone()),
					None => $crate::Node::Empty,
				}
			}
		}
	};
}

pub(crate) use getter;

macro_rules! setter {
	($name:ident -> bool) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: Option<bool>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::Bool(x))
				);
				self
			}
		}
	};

	($name:ident -> &str) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: Option<&str>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::String(x.to_string()))
				);
				self
			}
		}
	};

	($name:ident -> u64) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: Option<u64>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::Number(serde_json::Number::from(x)))
				);
				self
			}
		}
	};

	($name:ident -> i64) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: Option<i64>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::Number(serde_json::Number::from(x)))
				);
				self
			}
		}
	};

	($name:ident -> chrono::DateTime<chrono::Utc>) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::String(x.to_rfc3339()))
				);
				self
			}
		}
	};

	($name:ident -> node $t:ty ) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: $crate::Node<$t>) -> Self {
				$crate::macros::set_maybe_node(
					&mut self, stringify!($name), val
				);
				self
			}
		}
	};

	($name:ident::$rename:ident -> node $t:ty ) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: $crate::Node<$t>) -> Self {
				$crate::macros::set_maybe_node(
					&mut self, stringify!($rename), val
				);
				self
			}
		}
	};

	($name:ident -> type $t:ty ) => {
		paste::item! {
			fn [< set_$name:snake >](mut self, val: Option<$t>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, "type", val.map(|x| serde_json::Value::String(x.as_ref().to_string()))
				);
				self
			}
		}
	};
}

pub(crate) use setter;

#[cfg(feature = "unstructured")]
pub fn set_maybe_node(obj: &mut serde_json::Value, key: &str, node: crate::Node<serde_json::Value>) {
	if node.is_nothing() {
		set_maybe_value(obj, key, None)
	} else {
		set_maybe_value(obj, key, Some(node.into()))
	}
}

#[cfg(feature = "unstructured")]
pub fn set_maybe_value(obj: &mut serde_json::Value, key: &str, value: Option<serde_json::Value>) {
	if let Some(map) = obj.as_object_mut() {
		match value {
			Some(x) => map.insert(key.to_string(), x),
			None => map.remove(key),
		};
	} else {
		tracing::error!("error setting '{key}' on json Value: not an object");
	}
}
