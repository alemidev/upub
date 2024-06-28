use axum::{http::StatusCode, extract::State, Json};
use rand::Rng;
use sea_orm::{ActiveValue::{Set, NotSet}, ColumnTrait, Condition, EntityTrait, QueryFilter};
use upub::{traits::Administrable, Context};


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

fn token() -> String {
	// TODO should probably use crypto-safe rng
	rand::thread_rng()
		.sample_iter(&rand::distributions::Alphanumeric)
		.take(128)
		.map(char::from)
		.collect()
}

pub async fn login(
	State(ctx): State<Context>,
	Json(login): Json<LoginForm>
) -> crate::ApiResult<Json<AuthSuccess>> {
	// TODO salt the pwd
	match upub::model::credential::Entity::find()
		.filter(Condition::all()
			.add(upub::model::credential::Column::Login.eq(login.email))
			.add(upub::model::credential::Column::Password.eq(sha256::digest(login.password)))
			.add(upub::model::credential::Column::Active.eq(true))
		)
		.one(ctx.db())
		.await?
	{
		Some(x) => {
			let token = token();
			let expires = chrono::Utc::now() + chrono::Duration::hours(ctx.cfg().security.session_duration_hours);
			upub::model::session::Entity::insert(
				upub::model::session::ActiveModel {
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
		None => Err(crate::ApiError::unauthorized()),
	}
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RefreshForm {
	token: String,
}

pub async fn refresh(
	State(ctx): State<Context>,
	Json(login): Json<RefreshForm>
) -> crate::ApiResult<Json<AuthSuccess>> {
	if !ctx.cfg().security.allow_login_refresh {
		return Err(crate::ApiError::forbidden());
	}

	let prev = upub::model::session::Entity::find()
		.filter(upub::model::session::Column::Secret.eq(login.token))
		.one(ctx.db())
		.await?
		.ok_or_else(crate::ApiError::unauthorized)?;

	// allow refreshing tokens a little bit before they expire, specifically 1/4 of their lifespan before
	let quarter_session_lifespan = chrono::Duration::days(ctx.cfg().security.session_duration_hours) / 4;
	if prev.expires - quarter_session_lifespan > chrono::Utc::now() {
		return Ok(Json(AuthSuccess { token: prev.secret, user: prev.actor, expires: prev.expires }));
	}

	let token = token();
	let expires = chrono::Utc::now() + std::time::Duration::from_secs(3600 * 6);
	let user = prev.actor;
	let new_session = upub::model::session::ActiveModel {
		internal: NotSet,
		actor: Set(user.clone()),
		secret: Set(token.clone()),
		expires: Set(expires),
	};
	upub::model::session::Entity::insert(new_session)
		.exec(ctx.db())
		.await?;

	Ok(Json(AuthSuccess { token, expires, user }))
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
) -> crate::ApiResult<Json<String>> {
	if !ctx.cfg().security.allow_registration {
		return Err(crate::ApiError::forbidden());
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
