
#[axum::async_trait]
pub trait AnyQuery {
	async fn any(self, db: &sea_orm::DatabaseConnection) -> Result<bool, sea_orm::DbErr>;
}

#[axum::async_trait]
impl<T : sea_orm::EntityTrait> AnyQuery for sea_orm::Select<T> {
	async fn any(self, db: &sea_orm::DatabaseConnection) ->	Result<bool, sea_orm::DbErr> {
		Ok(self.one(db).await?.is_some())
	}
}

#[axum::async_trait]
impl<T : sea_orm::SelectorTrait + Send> AnyQuery for sea_orm::Selector<T> {
	async fn any(self, db: &sea_orm::DatabaseConnection) ->	Result<bool, sea_orm::DbErr> {
		Ok(self.one(db).await?.is_some())
	}
}
