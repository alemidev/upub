use reqwest::header::USER_AGENT;
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};

use crate::{VERSION, model};


#[derive(Debug, thiserror::Error)]
pub enum FetchError {
	#[error("could not dereference resource: {0}")]
	Network(#[from] reqwest::Error),

	#[error("error operating on database: {0}")]
	Database(#[from] sea_orm::DbErr),

	#[error("missing field when constructing object: {0}")]
	Field(#[from] model::FieldError),
}

pub struct Fetcher {
	db: DatabaseConnection,
	key: String, // TODO store pre-parsed
	domain: String, // TODO merge directly with Context so we don't need to copy this
}

impl Fetcher {
	pub fn new(db: DatabaseConnection, domain: String, key: String) -> Self {
		Fetcher { db, domain, key }
	}

	pub async fn user(&self, id: &str) -> Result<model::user::Model, FetchError> {
		if let Some(x) = model::user::Entity::find_by_id(id).one(&self.db).await? {
			return Ok(x); // already in db, easy
		}

		// TODO sign http fetches, we got the app key and db to get user keys just in case

		let user = reqwest::Client::new()
			.get(id)
			.header(USER_AGENT, format!("upub+{VERSION} ({})", self.domain)) // TODO put instance admin email
			.send()
			.await?
			.json::<serde_json::Value>()
			.await?;

		let user_model = model::user::Model::new(&user)?;

		model::user::Entity::insert(user_model.clone().into_active_model())
			.exec(&self.db).await?;

		Ok(user_model)
	}
}
