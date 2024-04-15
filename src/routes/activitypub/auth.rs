use axum::{http::StatusCode, extract::State, Json};
use rand::Rng;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{errors::UpubError, model, server::Context};


#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoginForm {
	email: String,
	password: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthSuccess {
	token: String,
	user: String,
	expires: chrono::DateTime<chrono::Utc>,
}

pub async fn login(State(ctx): State<Context>, Json(login): Json<LoginForm>) -> crate::Result<Json<AuthSuccess>> {
	// TODO salt the pwd
	match model::credential::Entity::find()
		.filter(Condition::all()
			.add(model::credential::Column::Email.eq(login.email))
			.add(model::credential::Column::Password.eq(sha256::digest(login.password)))
		)
		.one(ctx.db())
		.await?
	{
		Some(x) => {
			// TODO should probably use crypto-safe rng
			let token : String = rand::thread_rng()
				.sample_iter(&rand::distributions::Alphanumeric)
				.take(128)
				.map(char::from)
				.collect();
			let expires = chrono::Utc::now() + std::time::Duration::from_secs(3600 * 6);
			model::session::Entity::insert(
				model::session::ActiveModel {
					id: sea_orm::ActiveValue::Set(token.clone()),
					actor: sea_orm::ActiveValue::Set(x.id.clone()),
					expires: sea_orm::ActiveValue::Set(expires),
				}
			)
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(Json(AuthSuccess {
				token, expires,
				user: x.id
			}))
		},
		None => Err(UpubError::unauthorized()),
	}
}
