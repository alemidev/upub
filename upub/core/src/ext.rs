
#[async_trait::async_trait]
pub trait AnyQuery {
	async fn any(self, db: &sea_orm::DatabaseConnection) -> Result<bool, sea_orm::DbErr>;
}

#[async_trait::async_trait]
impl<T : sea_orm::EntityTrait> AnyQuery for sea_orm::Select<T> {
	async fn any(self, db: &sea_orm::DatabaseConnection) ->	Result<bool, sea_orm::DbErr> {
		Ok(self.one(db).await?.is_some())
	}
}

#[async_trait::async_trait]
impl<T : sea_orm::SelectorTrait + Send> AnyQuery for sea_orm::Selector<T> {
	async fn any(self, db: &sea_orm::DatabaseConnection) ->	Result<bool, sea_orm::DbErr> {
		Ok(self.one(db).await?.is_some())
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
