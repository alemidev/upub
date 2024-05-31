pub mod activitypub;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mastodon")]
pub mod mastodon;

pub mod builders;

#[cfg(not(feature = "mastodon"))]
pub mod mastodon {
	pub trait MastodonRouter {
		fn mastodon_routes(self) -> Self where Self: Sized { self }
	}
	
	impl MastodonRouter for axum::Router<upub::Context> {}
}

pub async fn serve(ctx: upub::Context, bind: String) -> upub::Result<()> {
	use activitypub::ActivityPubRouter;
	use mastodon::MastodonRouter;
	use tower_http::{cors::CorsLayer, trace::TraceLayer};

	let router = axum::Router::new()
		.ap_routes()
		.mastodon_routes() // no-op if mastodon feature is disabled
		.layer(CorsLayer::permissive())
		.layer(TraceLayer::new_for_http())
		.with_state(ctx);

	// run our app with hyper, listening locally on port 3000
	let listener = tokio::net::TcpListener::bind(bind)
		.await.expect("could not bind tcp socket");

	axum::serve(listener, router).await?;

	Ok(())
}
