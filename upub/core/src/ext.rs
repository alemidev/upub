use sea_orm::{ConnectionTrait, PaginatorTrait};


#[async_trait::async_trait]
pub trait AnyQuery {
	async fn any(self, db: &impl ConnectionTrait) -> Result<bool, sea_orm::DbErr>;
}

#[async_trait::async_trait]
impl<T : sea_orm::EntityTrait> AnyQuery for sea_orm::Select<T>
where
	T::Model : Sync,
{
	async fn any(self, db: &impl ConnectionTrait) -> Result<bool, sea_orm::DbErr> {
		Ok(self.count(db).await? > 0)
	}
}

#[async_trait::async_trait]
impl<T : sea_orm::SelectorTrait + Send + Sync> AnyQuery for sea_orm::Selector<T> {
	async fn any(self, db: &impl ConnectionTrait) -> Result<bool, sea_orm::DbErr> {
		Ok(self.count(db).await? > 0)
	}
}

pub trait LoggableError {
	fn info_failed(self, msg: &str);
	fn warn_failed(self, msg: &str);
	fn err_failed(self, msg: &str);
}

impl<T, E: std::error::Error> LoggableError for Result<T, E> {
	fn info_failed(self, msg: &str) {
		if let Err(e) = self {
			tracing::info!("{} : {}", msg, e);
		}
	}

	fn warn_failed(self, msg: &str) {
		if let Err(e) = self {
			tracing::warn!("{} : {}", msg, e);
		}
	}

	fn err_failed(self, msg: &str) {
		if let Err(e) = self {
			tracing::error!("{} : {}", msg, e);
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JsonVec<T>(pub Vec<T>);

impl<T> From<Vec<T>> for JsonVec<T> {
	fn from(value: Vec<T>) -> Self {
		JsonVec(value)
	}
}

impl<T> Default for JsonVec<T> {
	fn default() -> Self {
		JsonVec(Vec::new())
	}
}

impl<T: serde::de::DeserializeOwned> sea_orm::TryGetableFromJson for JsonVec<T> {}

impl<T: serde::ser::Serialize> std::convert::From<JsonVec<T>> for sea_orm::Value {
	fn from(source: JsonVec<T>) -> Self {
		sea_orm::Value::Json(serde_json::to_value(&source).ok().map(std::boxed::Box::new))
	}
}

impl<T: serde::de::DeserializeOwned + TypeName> sea_orm::sea_query::ValueType for JsonVec<T> {
	fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
		match v {
			sea_orm::Value::Json(Some(json)) => Ok(
				serde_json::from_value(*json).map_err(|_| sea_orm::sea_query::ValueTypeErr)?,
			),
			sea_orm::Value::Json(None) => Ok(JsonVec::default()),
			_ => Err(sea_orm::sea_query::ValueTypeErr),
		}
	}

	fn type_name() -> String {
		format!("JsonVec_{}", T::type_name())
	}

	fn array_type() -> sea_orm::sea_query::ArrayType {
		sea_orm::sea_query::ArrayType::Json
	}

	fn column_type() -> sea_orm::sea_query::ColumnType {
		sea_orm::sea_query::ColumnType::Json
	}
}

impl<T> sea_orm::sea_query::Nullable for JsonVec<T> {
	fn null() -> sea_orm::Value {
		sea_orm::Value::Json(None)
	}
}

pub trait TypeName {
	fn type_name() -> String;
}

impl TypeName for String {
	fn type_name() -> String {
		"String".to_string()
	}
}
