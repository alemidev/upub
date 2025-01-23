pub mod auth;
pub use auth::{AuthIdentity, Identity};

pub mod error;
pub use error::{ApiError, ApiResult};

pub mod builders;


#[cfg(feature = "activitypub")]
pub mod activitypub;

#[cfg(feature = "mastodon")]
pub mod mastodon;

#[cfg(feature = "web")]
pub mod web;


pub async fn serve(ctx: upub::Context, bind: String, shutdown: impl ShutdownToken) -> Result<(), std::io::Error> {
	use tower_http::{
		cors::CorsLayer, trace::TraceLayer, timeout::TimeoutLayer,
		classify::{SharedClassifier, StatusInRangeAsFailures}
	};

	let mut router = axum::Router::new();

	#[cfg(all(not(feature = "activitypub"), not(feature = "mastodon"), not(feature = "web")))] {
		compile_error!("at least one feature from ['activitypub', 'mastodon', 'web'] must be enabled");
	}

	#[cfg(feature = "activitypub")] { router = router.merge(activitypub::ap_routes(ctx.clone())); }
	#[cfg(feature = "mastodon")] { router = router.merge(mastodon::masto_routes(ctx.clone())); }
	#[cfg(feature = "web")] { router = router.merge(web::web_routes(ctx.clone())); }

	router = router
		.layer(
			tower::ServiceBuilder::new()
				// TODO 4xx errors aren't really failures but since upub is in development it's useful to log
				//      these too, in case something's broken
				.layer(
					TraceLayer::new(SharedClassifier::new(StatusInRangeAsFailures::new(400..=999)))
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
				.layer(TimeoutLayer::new(std::time::Duration::from_secs(ctx.cfg().security.request_timeout)))
		);

	tracing::info!("serving api routes on {bind}");

	let listener = tokio::net::TcpListener::bind(bind).await?;
	axum::serve(listener, router)
		.with_graceful_shutdown(shutdown.event())
		.await?;

	Ok(())
}


pub trait ShutdownToken: Sync + Send + 'static {
	//                TODO this is bs...
	fn event(self) -> impl std::future::Future<Output = ()> + std::marker::Send;
}
