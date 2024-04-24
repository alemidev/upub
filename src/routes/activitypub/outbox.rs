use axum::{extract::{Query, State}, http::StatusCode, Json};

use crate::{errors::UpubError, routes::activitypub::{CreationResult, JsonLD, Pagination}, server::{auth::AuthIdentity, Context}, url};

pub async fn get(State(ctx): State<Context>) -> crate::Result<JsonLD<serde_json::Value>> {
	crate::server::builders::collection(&url!(ctx, "/outbox"), None)
}

pub async fn page(
	State(ctx): State<Context>,
	Query(page): Query<Pagination>,
	AuthIdentity(auth): AuthIdentity,
) -> crate::Result<JsonLD<serde_json::Value>> {
	crate::server::builders::paginate(
		url!(ctx, "/outbox/page"),
		auth.filter_condition(), // TODO filter local only stuff
		ctx.db(),
		page,
	)
		.await
}

pub async fn post(
	State(_ctx): State<Context>,
	AuthIdentity(_auth): AuthIdentity,
	Json(_activity): Json<serde_json::Value>,
) -> Result<CreationResult, UpubError> {
	// TODO administrative actions may be carried out against this outbox?
	Err(StatusCode::NOT_IMPLEMENTED.into())
}
