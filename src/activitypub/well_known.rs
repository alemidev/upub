use axum::{extract::{Query, State}, http::StatusCode, response::{IntoResponse, Response}};
use jrd::{JsonResourceDescriptor, JsonResourceDescriptorLink};
use sea_orm::EntityTrait;

use crate::{server::Context, model};

#[derive(Debug, serde::Deserialize)]
pub struct WebfingerQuery {
	pub resource: String,
}

pub struct JsonRD<T>(pub T);
impl<T: serde::Serialize> IntoResponse for JsonRD<T> {
	fn into_response(self) -> Response {
		([("Content-Type", "application/jrd+json")], axum::Json(self.0)).into_response()
	}
}

pub async fn webfinger(State(ctx): State<Context>, Query(query): Query<WebfingerQuery>) -> Result<JsonRD<JsonResourceDescriptor>, StatusCode> {
	if let Some((user, domain)) = query.resource.split_once('@') {
		let uid = ctx.uid(user.to_string());
		match model::user::Entity::find_by_id(uid)
			.one(ctx.db())
			.await
		{
			Ok(Some(x)) => Ok(JsonRD(JsonResourceDescriptor {
				subject: format!("acct:{user}@{domain}"),
				aliases: vec![x.id.clone()],
				links: vec![
					JsonResourceDescriptorLink {
						rel: "self".to_string(),
						link_type: Some("application/ld+json".to_string()),
						href: Some(x.id),
						properties: jrd::Map::default(),
						titles: jrd::Map::default(),
					},
				],
				expires: None,
				properties: jrd::Map::default(),
			})),
			Ok(None) => Err(StatusCode::NOT_FOUND),
			Err(e) => {
				tracing::error!("error executing webfinger query: {e}");
				Err(StatusCode::INTERNAL_SERVER_ERROR)
			},
		}
	} else {
		Err(StatusCode::UNPROCESSABLE_ENTITY)
	}
}

// i don't even want to bother with XML, im just returning a formatted xml string
pub async fn host_meta(State(ctx): State<Context>) -> Response {
	(
		[("Content-Type", "application/xrd+xml")],
		format!(r#"<?xml version="1.0" encoding="UTF-8"?>
			<XRD xmlns="http://docs.oasis-open.org/ns/xri/xrd-1.0">
				<Link type="application/xrd+xml" template="{}/.well-known/webfinger?resource={{uri}}" rel="lrdd" />
			</XRD>"#,
			ctx.base())
	).into_response()
}
