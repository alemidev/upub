use axum::{http::StatusCode, extract::State, Json};
use rand::Rng;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{errors::UpubError, model, server::{admin::Administrable, Context}};


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

pub async fn login(
	State(ctx): State<Context>,
	Json(login): Json<LoginForm>
) -> crate::Result<Json<AuthSuccess>> {
	// TODO salt the pwd
	match model::credential::Entity::find()
		.filter(Condition::all()
			.add(model::credential::Column::Login.eq(login.email))
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
					internal: sea_orm::ActiveValue::NotSet,
					secret: sea_orm::ActiveValue::Set(token.clone()),
					actor: sea_orm::ActiveValue::Set(x.actor.clone()),
					expires: sea_orm::ActiveValue::Set(expires),
				}
			)
				.exec(ctx.db())
				.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
			Ok(Json(AuthSuccess {
				token, expires,
				user: x.actor
			}))
		},
		None => Err(UpubError::unauthorized()),
	}
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RegisterForm {
	username: String,
	password: String,
	display_name: Option<String>,
	summary: Option<String>,
	avatar_url: Option<String>,
	banner_url: Option<String>,
}

pub async fn register(
	State(ctx): State<Context>,
	Json(registration): Json<RegisterForm>
) -> crate::Result<Json<String>> {
	if !ctx.cfg().security.allow_registration {
		return Err(UpubError::forbidden());
	}

	ctx.register_user(
		registration.username.clone(),
		registration.password,
		registration.display_name,
		registration.summary,
		registration.avatar_url,
		registration.banner_url
	).await?;

	Ok(Json(ctx.uid(&registration.username)))
}
