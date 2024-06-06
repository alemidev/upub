use tower_http::classify::{SharedClassifier, StatusInRangeAsFailures};

pub mod auth;
pub use auth::{AuthIdentity, Identity};

pub mod error;
pub use error::{ApiError, ApiResult};

pub mod activitypub;

#[cfg(feature = "mastodon")]
pub mod mastodon;

#[cfg(feature = "web")]
pub mod web;

pub mod builders;

#[cfg(not(feature = "mastodon"))]
pub mod mastodon {
	pub trait MastodonRouter {
		fn mastodon_routes(self) -> Self where Self: Sized { self }
	}
	
	impl MastodonRouter for axum::Router<upub::Context> {}
}

pub async fn serve(ctx: upub::Context, bind: String) -> Result<(), std::io::Error> {
	use activitypub::ActivityPubRouter;
	use mastodon::MastodonRouter;
	use tower_http::{cors::CorsLayer, trace::TraceLayer};

	let router = axum::Router::new()
		.layer(
			// TODO 4xx errors aren't really failures but since upub is in development it's useful to log
			//      these too, in case something's broken
			TraceLayer::new(SharedClassifier::new(StatusInRangeAsFailures::new(300..=999)))
				.make_span_with(|req: &axum::http::Request<_>| {
					tracing::span!(
						tracing::Level::INFO,
						"request",
						uri = %req.uri(),
						status_code = tracing::field::Empty,
					)
				})
		)
		.layer(CorsLayer::permissive())
		.ap_routes()
		.mastodon_routes() // no-op if mastodon feature is disabled
		.with_state(ctx);

	// run our app with hyper, listening locally on port 3000
	let listener = tokio::net::TcpListener::bind(bind).await?;

	axum::serve(listener, router).await?;

	Ok(())
}
