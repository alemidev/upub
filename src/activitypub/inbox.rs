use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use crate::{model, server::Context};

use super::JsonLD;


pub async fn get(State(ctx) : State<Context>, Json(_object): Json<serde_json::Value>) -> Result<JsonLD<serde_json::Value>, StatusCode> {
	match model::activity::Entity::find()
		.order_by(model::activity::Column::Published, sea_orm::Order::Desc)
		.limit(20)
		.all(ctx.db())
		.await
	{
		Ok(x) => todo!(),
		Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
	}
}

