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
