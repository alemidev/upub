#[derive(Debug, thiserror::Error)]
#[error("invalid type value")]
pub struct TypeValueError;

impl From<TypeValueError> for sea_orm::sea_query::ValueTypeErr {
	fn from(_: TypeValueError) -> Self {
		sea_orm::sea_query::ValueTypeErr
	}
}

impl From<TypeValueError> for sea_orm::TryGetError {
	fn from(_: TypeValueError) -> Self {
		sea_orm::TryGetError::Null("value is not a valid type".into())
	}
}

#[macro_export]
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
				type Error = $crate::activitystream::types::TypeValueError;

				fn try_from(value:&str) -> Result<Self, Self::Error> {
					match value {
						$(stringify!($flat) => Ok(Self::$flat),)*
						_ => {
							$(
								if let Ok(x) = $inner::try_from(value) {
									return Ok(Self::$deep(x));
								}
							)*
							Err($crate::activitystream::types::TypeValueError)
						},
					}
				}
			}

			impl From<$enum_name> for sea_orm::Value {
				fn from(value: $enum_name) -> sea_orm::Value {
					sea_orm::Value::String(Some(Box::new(value.as_ref().to_string())))
				}
			}

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
			
			impl sea_orm::TryGetable for $enum_name {
				fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::prelude::QueryResult, index: I) -> Result<Self, sea_orm::TryGetError> {
					let x : String = res.try_get_by(index)?;
					Ok(Self::try_from(x.as_str())?)
				}
			}
		)*
	};
}
