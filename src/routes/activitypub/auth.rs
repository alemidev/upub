use axum::{http::StatusCode, extract::State, Json};
use rand::Rng;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{model, server::Context};


#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoginForm {
	email: String,
	password: String,
}

pub async fn login(State(ctx): State<Context>, Json(login): Json<LoginForm>) -> Result<Json<serde_json::Value>, StatusCode> {
	// TODO salt the pwd
	match model::credential::Entity::find()
		.filter(Condition::all()
			.add(model::credential::Column::Email.eq(login.email))
			.add(model::credential::Column::Password.eq(sha256::digest(login.password)))
		)
		.one(ctx.db())
		.await
	{
		Ok(Some(x)) => {
			// TODO should probably use crypto-safe rng
			let token : String = rand::thread_rng()
				.sample_iter(&rand::distributions::Alphanumeric)
				.take(128)
				.map(char::from)
				.collect();
			model::session::Entity::insert(
				model::session::ActiveModel {
					id: sea_orm::ActiveValue::Set(token.clone()),
					actor: sea_orm::ActiveValue::Set(x.id),
					expires: sea_orm::ActiveValue::Set(chrono::Utc::now() + std::time::Duration::from_secs(3600 * 6)),
				}
			)
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(Json(serde_json::Value::String(token)))
		},
		Ok(None) => Err(StatusCode::UNAUTHORIZED),
		Err(e) => {
			tracing::error!("error querying db for user credentials: {e}");
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}
