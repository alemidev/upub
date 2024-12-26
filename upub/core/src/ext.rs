use sea_orm::{ConnectionTrait, PaginatorTrait};

pub trait IntoActivityPub {
	fn into_activity_pub_json(self, ctx: &crate::Context) -> serde_json::Value;
}

#[allow(async_fn_in_trait)]
pub trait AnyQuery {
	async fn any(self, db: &impl ConnectionTrait) -> Result<bool, sea_orm::DbErr>;
}

impl<T : sea_orm::EntityTrait> AnyQuery for sea_orm::Select<T>
where
	T::Model : Sync,
{
	async fn any(self, db: &impl ConnectionTrait) -> Result<bool, sea_orm::DbErr> {
		// TODO ConnectionTrait became an iterator?? self.count(db) gives error now
		Ok(PaginatorTrait::count(self, db).await? > 0)
	}
}

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

// TODO we need this dummy to access the default implementation, which needs to be wrapped to catch
//      nulls. is there a way to directly call super::try_get_from_json ?? i think this gets
//      compiled into a lot of variants...
#[derive(serde::Deserialize)]
struct DummyVec<T>(pub Vec<T>);
impl<T: serde::de::DeserializeOwned> sea_orm::TryGetableFromJson for DummyVec<T> {}

impl<T: serde::de::DeserializeOwned> sea_orm::TryGetableFromJson for JsonVec<T> {
	fn try_get_from_json<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, idx: I) -> Result<Self, sea_orm::TryGetError> {
		match DummyVec::try_get_from_json(res, idx) {
			Ok(DummyVec(x)) => Ok(Self(x)),
			Err(sea_orm::TryGetError::Null(_)) => Ok(Self::default()),
			Err(e) => Err(e),
		}
	}

	fn from_json_vec(value: serde_json::Value) -> Result<Vec<Self>, sea_orm::TryGetError> {
		match DummyVec::from_json_vec(value) {
			Ok(x) => Ok(x.into_iter().map(|x| JsonVec(x.0)).collect()),
			Err(sea_orm::TryGetError::Null(_)) => Ok(vec![]),
			Err(e) => Err(e),
		}
	}
}

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
