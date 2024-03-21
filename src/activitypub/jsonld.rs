// got this from https://github.com/kitsune-soc/kitsune/blob/b023a12b687dd9a274233a5a9950f2de5e192344/kitsune/src/http/responder.rs
// i was trying to do it with middlewares but this is way cleaner

use axum::{
	response::{IntoResponse, Response},
	Json,
};
use serde::Serialize;

pub struct JsonLD<T>(pub T);

impl<T> IntoResponse for JsonLD<T>
where
	T: Serialize,
{
	fn into_response(self) -> Response {
		(
			[(
				"Content-Type",
				"application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"",
			)],
			Json(self.0),
		)
			.into_response()
	}
}
