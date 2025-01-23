pub mod accounts;
pub mod instance;

use axum::{http::StatusCode, routing::{delete, get, patch, post}, Router};
use crate::server::Context;

async fn todo() -> StatusCode { StatusCode::NOT_IMPLEMENTED }

pub fn masto_routes(ctx: upub::Context) -> Router {
	use crate::routes::mastodon as mas;
	Router::new().nest(
		// TODO Oauth is just under /oauth
		"/api/v1", Router::new()
			.route("/apps", post(todo)) // create an application
			.route("/apps/verify_credentials", post(todo)) // confirm that the app's oauth2 credentials work
			.route("/emails/confirmations", post(todo))
			.route("/accounts", post(todo))
			.route("/accounts/verify_credentials", get(todo))
			.route("/accounts/update_credentials", patch(todo))
			.route("/accounts/:id", get(mas::accounts::view))
			.route("/accounts/:id/statuses", get(todo))
			.route("/accounts/:id/followers", get(todo))
			.route("/accounts/:id/following", get(todo))
			.route("/accounts/:id/featured_tags", get(todo))
			.route("/accounts/:id/lists", get(todo))
			.route("/accounts/:id/follow", post(todo))
			.route("/accounts/:id/unfollow", post(todo))
			.route("/accounts/:id/remove_from_followers", post(todo))
			.route("/accounts/:id/block", post(todo))
			.route("/accounts/:id/unblock", post(todo))
			.route("/accounts/:id/mute", post(todo))
			.route("/accounts/:id/unmute", post(todo))
			.route("/accounts/:id/pin", post(todo))
			.route("/accounts/:id/unpin", post(todo))
			.route("/accounts/:id/note", post(todo))
			.route("/accounts/relationships", get(todo))
			.route("/accounts/familiar_followers", get(todo))
			.route("/accounts/search", get(todo))
			.route("/accounts/lookup", get(todo))
			.route("/accounts/:id/identity_proofs", get(todo))
			.route("/bookmarks", get(todo))
			.route("/favourites", get(todo))
			.route("/mutes", get(todo))
			.route("/blocks", get(todo))
			.route("/domain_blocks", get(todo))
			.route("/domain_blocks", post(todo))
			.route("/domain_blocks", delete(todo))
			// TODO filters! api v2
			.route("/reports", post(todo))
			.route("/follow_requests", get(todo))
			.route("/follow_requests/:account_id/authorize", get(todo))
			.route("/follow_requests/:account_id/reject", get(todo))
			.route("/endorsements", get(todo))
			.route("/featured_tags", get(todo))
			.route("/featured_tags", post(todo))
			.route("/featured_tags/:id", delete(todo))
			.route("/featured_tags/suggestions", get(todo))
			.route("/preferences", get(todo))
			.route("/followed_tags", get(todo))
			// TODO suggestions! api v2
			.route("/suggestions", get(todo))
			.route("/suggestions/:account_id", delete(todo))
			.route("/tags/:id", get(todo))
			.route("/tags/:id/follow", post(todo))
			.route("/tags/:id/unfollow", post(todo))
			.route("/profile/avatar", delete(todo))
			.route("/profile/header", delete(todo))
			.route("/statuses", post(todo))
			// ...
			.route("/instance", get(mas::instance::get))
	)
		.with_state(ctx)
}
