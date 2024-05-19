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
		fn $name(&self) -> Option<$t> {
			self.get("type")?.as_str()?.try_into().ok()
		}
	};

	($name:ident -> bool) => {
		fn $name(&self) -> Option<bool> {
			self.get(stringify!($name))?.as_bool()
		}
	};

	($name:ident -> &str) => {
		fn $name(&self) -> Option<&str> {
			self.get(stringify!($name))?.as_str()
		}
	};

	($name:ident::$rename:ident -> bool) => {
		fn $name(&self) -> Option<bool> {
			self.get(stringify!($rename))?.as_bool()
		}
	};

	($name:ident::$rename:ident -> &str) => {
		fn $name(&self) -> Option<&str> {
			self.get(stringify!($rename))?.as_str()
		}
	};

	($name:ident -> f64) => {
		fn $name(&self) -> Option<f64> {
			self.get(stringify!($name))?.as_f64()
		}
	};

	($name:ident::$rename:ident -> f64) => {
		fn $name(&self) -> Option<f64> {
			self.get(stringify!($rename))?.as_f64()
		}
	};

	($name:ident -> u64) => {
		fn $name(&self) -> Option<u64> {
			self.get(stringify!($name))?.as_u64()
		}
	};

	($name:ident::$rename:ident -> u64) => {
		fn $name(&self) -> Option<u64> {
			self.get(stringify!($rename))?.as_u64()
		}
	};

	($name:ident -> chrono::DateTime<chrono::Utc>) => {
		fn $name(&self) -> Option<chrono::DateTime<chrono::Utc>> {
			Some(
				chrono::DateTime::parse_from_rfc3339(
						self
							.get(stringify!($name))?
							.as_str()?
					)
					.ok()?
					.with_timezone(&chrono::Utc)
			)
		}
	};

	($name:ident::$rename:ident -> chrono::DateTime<chrono::Utc>) => {
		fn $name(&self) -> Option<chrono::DateTime<chrono::Utc>> {
			Some(
				chrono::DateTime::parse_from_rfc3339(
						self
							.get(stringify!($rename))?
							.as_str()?
					)
					.ok()?
					.with_timezone(&chrono::Utc)
			)
		}
	};

	($name:ident -> node $t:ty) => {
		fn $name(&self) -> $crate::Node<$t> {
			match self.get(stringify!($name)) {
				Some(x) => $crate::Node::from(x.clone()),
				None => $crate::Node::Empty,
			}
		}
	};

	($name:ident::$rename:ident -> node $t:ty) => {
		fn $name(&self) -> $crate::Node<$t> {
			match self.get(stringify!($rename)) {
				Some(x) => $crate::Node::from(x.clone()),
				None => $crate::Node::Empty,
			}
		}
	};
}

pub(crate) use getter;

macro_rules! setter {
	($name:ident -> bool) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<bool>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::Bool(x))
				);
				self
			}
		}
	};

	($name:ident::$rename:ident -> bool) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<bool>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($rename), val.map(|x| serde_json::Value::Bool(x))
				);
				self
			}
		}
	};

	($name:ident -> &str) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<&str>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::String(x.to_string()))
				);
				self
			}
		}
	};

	($name:ident::$rename:ident -> &str) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<&str>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($rename), val.map(|x| serde_json::Value::String(x.to_string()))
				);
				self
			}
		}
	};

	($name:ident -> u64) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<u64>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::Number(serde_json::Number::from(x)))
				);
				self
			}
		}
	};

	($name:ident::$rename:ident -> u64) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<u64>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($rename), val.map(|x| serde_json::Value::Number(serde_json::Number::from(x)))
				);
				self
			}
		}
	};

	($name:ident -> chrono::DateTime<chrono::Utc>) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($name), val.map(|x| serde_json::Value::String(x.to_rfc3339()))
				);
				self
			}
		}
	};

	($name:ident::$rename:ident -> chrono::DateTime<chrono::Utc>) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<chrono::DateTime<chrono::Utc>>) -> Self {
				$crate::macros::set_maybe_value(
					&mut self, stringify!($rename), val.map(|x| serde_json::Value::String(x.to_rfc3339()))
				);
				self
			}
		}
	};

	($name:ident -> node $t:ty ) => {
		paste::item! {
			fn [< set_$name >](mut self, val: $crate::Node<$t>) -> Self {
				$crate::macros::set_maybe_node(
					&mut self, stringify!($name), val
				);
				self
			}
		}
	};

	($name:ident::$rename:ident -> node $t:ty ) => {
		paste::item! {
			fn [< set_$name >](mut self, val: $crate::Node<$t>) -> Self {
				$crate::macros::set_maybe_node(
					&mut self, stringify!($rename), val
				);
				self
			}
		}
	};

	($name:ident -> type $t:ty ) => {
		paste::item! {
			fn [< set_$name >](mut self, val: Option<$t>) -> Self {
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
	match node {
		crate::Node::Object(x) => {
			set_maybe_value(
				obj, key, Some(*x),
			);
		},
		crate::Node::Link(l) => {
			set_maybe_value(
				obj, key, Some(serde_json::Value::String(l.href().to_string())),
			);
		},
		crate::Node::Array(_) => {
			set_maybe_value(
				obj, key, Some(serde_json::Value::Array(node.into_iter().collect())),
			);
		},
		crate::Node::Empty => {
			set_maybe_value(
				obj, key, None,
			);
		},
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

#[cfg(feature = "unstructured")]
pub(crate) trait InsertValue {
	fn insert_node(&mut self, k: &str, v: crate::Node<serde_json::Value>);
	fn insert_str(&mut self, k: &str, v: Option<&str>);
	fn insert_float(&mut self, k: &str, f: Option<f64>);
	fn insert_timestr(&mut self, k: &str, t: Option<chrono::DateTime<chrono::Utc>>);
}

#[cfg(feature = "unstructured")]
impl InsertValue for serde_json::Map<String, serde_json::Value> {
	fn insert_node(&mut self, k: &str, node: crate::Node<serde_json::Value>) {
		match node {
			crate::Node::Object(x) => {
				self.insert(
					k.to_string(),
					*x,
				);
			},
			crate::Node::Array(ref _arr) => {
				self.insert(
					k.to_string(),
					serde_json::Value::Array(node.into_iter().collect()),
				);
			},
			crate::Node::Link(l) => {
				self.insert(
					k.to_string(),
					serde_json::Value::String(l.href().to_string()),
				);
			},
			crate::Node::Empty => {},
		};
	}

	fn insert_str(&mut self, k: &str, v: Option<&str>) {
		if let Some(v) = v {
			self.insert(
				k.to_string(),
				serde_json::Value::String(v.to_string()),
			);
		}
	}

	fn insert_float(&mut self, k: &str, v: Option<f64>) {
		if let Some(v) = v {
			if let Some(n) = serde_json::Number::from_f64(v) {
				self.insert(
					k.to_string(),
					serde_json::Value::Number(n),
				);
			}
		}
	}

	fn insert_timestr(&mut self, k: &str, t: Option<chrono::DateTime<chrono::Utc>>) {
		if let Some(published) = t {
			self.insert(
				k.to_string(),
				serde_json::Value::String(published.to_rfc3339()),
			);
		}
	}
}
